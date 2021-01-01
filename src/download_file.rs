use reqwest;
use std::{fs::File, io::Seek, time::Instant};
use std::path::Path;


use std::io::Write;

use std::io::SeekFrom;
use std::{thread, time};

use crate::{
    tor_share_url::TorShareUrl,
    tor_utils::{start_tor_socks5, TorDirOptions, TorDirectory, TorSocks5},
};
use error_chain::error_chain;
use reqwest::header::CONTENT_LENGTH;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct DownloadOptions {
    #[structopt(flatten)]
    pub tor_dir_options: TorDirOptions,
    #[structopt(parse(try_from_str = TorShareUrl::from_str))]
    pub url: TorShareUrl,
}

pub struct FileInformation {
    pub name: String,
    pub size: f64,
}

pub struct DownloadProgress {
    pub downloaded_megabytes: f64,
    pub percent: f32,
    pub speed: f64,
}

pub enum DownloadState<'a> {
    ConnectingWaitingForTor,
    ConnectingWaitingForProxy(&'a TorSocks5),
    ConnectedWaitingForPeer,
    ConnectedAskForContinue,
    ConnectedRetrievingFileInformation,
    ConnectedRetrievedFileInformation(&'a FileInformation),
    ConnectedDownloading(&'a FileInformation, DownloadProgress),
    DisconnectedError(String),
}

pub enum DownloadStateReturn {
    Continue(bool)
}

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
         ParseIntError(std::num::ParseIntError);
         ToStrError(reqwest::header::ToStrError);
     }
}
pub async fn download_file(download_options: &DownloadOptions, cb: impl Fn(DownloadState) -> Option<DownloadStateReturn>) {
    let tor_dir = TorDirectory::from_general_options(&download_options.tor_dir_options);
    let tor_socks5 = TorSocks5::from_random_port();
    let tor_share_url = &download_options.url;
    cb(DownloadState::ConnectingWaitingForTor);
    
    start_tor_socks5(&tor_dir, &tor_socks5);
    println!("asdasdasd");

    let socks5_url = tor_socks5.to_string();
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&socks5_url).unwrap())
        .build()
        .unwrap();

    let url = tor_share_url.to_url();
    loop {
        let result = client.head(&url).send().await;
        if let Err(e) = result {
            //println!("{}\n", e);
            let socks5_unreachable = e.to_string().contains("Proxy server unreachable");
            if socks5_unreachable {
                cb(DownloadState::ConnectingWaitingForProxy(&tor_socks5));
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
        let file_name = if let Some(content_disposition) = result.headers().get("Content-Disposition") {
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

        let file_name: String = file_name.unwrap_or_else(|| format!("{}.file", tor_share_url.path));
        let file_path: &Path = Path::new(&file_name);

        
        let continue_download = if file_path.exists() {
            if !file_path.is_file() {
                cb(DownloadState::DisconnectedError("something on this path \"{}\" already exists and is not a file where we can continue the download".into()));
                return;
            }

            let can_continue = if let Some(downloaded_state_return) = cb(DownloadState::ConnectedAskForContinue) {
                if let DownloadStateReturn::Continue(can_continue) = downloaded_state_return {
                    can_continue
                } else {
                    println!("Warning: Received a wrong state type from callback"); 
                    false
                }
            } else {
                false
            };

            if !can_continue {
                return;
            }

            true 
        } else {
            false
        };



        let (mut dest, mut downloaded_bytes) = if continue_download {
            let mut dest = File::open(&file_name).unwrap();
            let continue_position: usize = dest.seek(SeekFrom::End(0)).unwrap() as usize;
            (dest, continue_position)
        } else {
            (File::create(&file_name).unwrap(), 0 as usize)
        };

        let file_size: f64 = result
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or("0")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .parse::<f64>()
            .unwrap();

        let file_information = FileInformation {
            name: file_name,
            size: file_size,
        };

        cb(DownloadState::ConnectedRetrievedFileInformation(
            &file_information,
        ));




        let result = client.get(&url);
        let result = if continue_download {
            println!("Continuing download {}/{}", downloaded_bytes, file_information.size);
            result.header("Content-Range", format!("bytes {}-{}/{}", downloaded_bytes, file_information.size, file_information.size))
        } else {
            result
        };
        
        let result = result.send().await;
        if let Err(e) = result {
            //println!("{}\n", e);
            let socks5_unreachable = e.to_string().contains("Proxy server unreachable");
            if socks5_unreachable {
                cb(DownloadState::ConnectingWaitingForProxy(&tor_socks5));
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

        let mut last_write = Instant::now();
        // bytes per second
        let mut speed: f64 = -1.0;
        let mut downloaded_bytes_last_second = 0;
        loop {
            let chunk = result.chunk().await;
            if let Err(e) = chunk {
                cb(DownloadState::DisconnectedError(e.to_string()));
                break;
            }
            let chunk = chunk.unwrap();
            if chunk.is_none() {
                break;
            }
            let chunk = chunk.unwrap();
            dest.write(&chunk);
            let elapsed_time_as_secs = last_write.elapsed().as_secs_f64();

            downloaded_bytes_last_second = downloaded_bytes_last_second + chunk.len();
            if elapsed_time_as_secs > 0.5 {
                speed = downloaded_bytes_last_second as f64 / 1000000.0 / elapsed_time_as_secs;
                downloaded_bytes_last_second = 0;
                last_write = Instant::now();
            } else if speed == -1.0 {
                speed = downloaded_bytes_last_second as f64 / 1000000.0 / elapsed_time_as_secs;
            }

            downloaded_bytes = downloaded_bytes + chunk.len();

            let downloaded_megabytes = downloaded_bytes as f64 / 1000000.0;
            if file_size == 0.0 {
                cb(DownloadState::ConnectedDownloading(
                    &file_information,
                    DownloadProgress {
                        downloaded_megabytes,
                        percent: -1.0,
                        speed,
                    },
                ));
            } else {
                let percent = downloaded_megabytes as f32 / file_size as f32 * 100.0;

                cb(DownloadState::ConnectedDownloading(
                    &file_information,
                    DownloadProgress {
                        downloaded_megabytes,
                        percent,
                        speed,
                    },
                ));
            }
        }
        println!("\n");
        break;
    }
}
