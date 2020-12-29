use reqwest;
use termion::cursor::Down;
use std::{io::Write, rc::Rc};
use std::{
    fs::File,
    time::{Instant, SystemTime},
};
use std::{thread, time};

use crate::{cli::{print_status_line, save_cursor_position, Color}, tor_share_url::{self, TorShareUrl}, tor_utils::{TorSocks5, start_tor_socks5}};
use crate::SOCKS5_PORT;
use error_chain::error_chain;
use reqwest::header::CONTENT_LENGTH;



pub struct FileInformation {
    pub name: String,
    pub size: f64
}

pub struct DownloadProgress {
    pub downloaded_megabytes: f64,
    pub percent: f32,
    pub speed: f64
}

pub enum DownloadState {
    ConnectingWaitingForTor,
    ConnectingWaitingForProxy,
    ConnectedWaitingForPeer,
    ConnectedRetrievingFileInformation,
    ConnectedRetrievedFileInformation(Rc<FileInformation>),
    ConnectedDownloading(Rc<FileInformation>, DownloadProgress),
    DisconnectedError(String)
}


error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
         ParseIntError(std::num::ParseIntError);
         ToStrError(reqwest::header::ToStrError);
     }
}
pub async fn download_file(tor_socks5: TorSocks5, tor_share_url: TorShareUrl, cb: fn(DownloadState)) {
    cb(DownloadState::ConnectingWaitingForTor);
    let socks5_url = tor_socks5.to_string();
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&socks5_url).unwrap())
        .build().unwrap();

    let url = tor_share_url.to_url();
    println!("{}\n", url);
    loop {
        let result = client.get(&url).send().await;
        if let Err(e) = result {
            let host_offline = e.to_string().contains("Host unreachable");
            if !host_offline {
                cb(DownloadState::ConnectingWaitingForProxy);
                thread::sleep(time::Duration::from_millis(50));
                continue;
            } else {
                cb(DownloadState::ConnectedWaitingForPeer);
                thread::sleep(time::Duration::from_millis(50));
                continue;
            };
        }
        cb(DownloadState::ConnectedRetrievingFileInformation);

        let mut result = result.unwrap();
        let fname = if let Some(content_disposition) = result.headers().get("Content-Disposition") {
            let content_disposition: String = content_disposition.to_str().unwrap().into();
            if let Some(filename_index) = content_disposition.rfind("filename=\"") {
                Some(
                    content_disposition
                        .chars()
                        .skip(filename_index + 10)
                        .take_while(|x| *x != '"')
                        .collect(),
                )
            } else {
                None
            }
        } else {
            None
        };

        let fname: String = fname.unwrap_or_else(|| format!("{}.file", tor_share_url.path));
        let mut dest = File::create(&fname).unwrap();

        let file_size: f64 = result
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or("0").unwrap()
            .to_str().unwrap()
            .to_string()
            .parse::<f64>()
            .unwrap()
            / 1000000.0;

        let file_information = Rc::new(FileInformation { name: fname, size: file_size });

        cb(DownloadState::ConnectedRetrievedFileInformation(Rc::clone(&file_information)));

        let mut downloaded_megabytes: f64 = 0.0;
        let mut last_write = Instant::now();
        // bytes per second
        let mut speed: f64 = -1.0;
        let mut downloaded_bytes_last_second = 0;
        while let Some(chunk) = result.chunk().await.unwrap() {
            let elapsed_time_as_secs = last_write.elapsed().as_secs_f64();

            downloaded_bytes_last_second = downloaded_bytes_last_second + chunk.len();
            if elapsed_time_as_secs > 0.5 {
                speed = downloaded_bytes_last_second as f64 / 1000000.0 / elapsed_time_as_secs;
                downloaded_bytes_last_second = 0;
                last_write = Instant::now();
            } else if speed == -1.0 {
                speed = downloaded_bytes_last_second as f64 / 1000000.0 / elapsed_time_as_secs;
            }
            let chunk_size_as_megabyte = chunk.len() as f64 / 1000000.0;
            let megabytes_per_second = chunk_size_as_megabyte / elapsed_time_as_secs / 10.0;

            downloaded_megabytes = downloaded_megabytes + chunk_size_as_megabyte;
            if file_size == 0.0 {
                cb(DownloadState::ConnectedDownloading(Rc::clone(&file_information), DownloadProgress { downloaded_megabytes, percent: -1.0, speed}));

            } else {
                let percent = downloaded_megabytes as f32 / file_size as f32 * 100.0;

                cb(DownloadState::ConnectedDownloading(Rc::clone(&file_information), DownloadProgress { downloaded_megabytes, percent, speed}));
            }
        }
        println!("\n");
        break;
    }
}
