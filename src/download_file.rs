use reqwest;
use std::io::Write;
use std::{
    fs::File,
    time::{Instant, SystemTime},
};
use std::{thread, time};

use crate::cli::{print_status_line, save_cursor_position, Color};
use crate::SOCKS5_PORT;
use error_chain::error_chain;
use reqwest::header::CONTENT_LENGTH;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
         ParseIntError(std::num::ParseIntError);
         ToStrError(reqwest::header::ToStrError);
     }
}
pub async fn download_file(hidden_service: String, path: String) -> Result<()> {
    let socks5_url = format!("socks5h://127.0.0.1:{}", SOCKS5_PORT);
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&socks5_url)?)
        .build()?;
    save_cursor_position();
    print_status_line(&Color::Yellow, "Connecting to tor network...");

    let url = format!("http://{}/{}", hidden_service, path);
    loop {
        let result = client.get(&url).send().await;
        if let Err(e) = result {
            let host_offline = e.to_string().contains("Host unreachable");
            if !host_offline {
                print_status_line(
                    &Color::Yellow,
                    "Connecting to tor network... Waiting for proxy...",
                );
                thread::sleep(time::Duration::from_millis(50));
                continue;
            } else {
                print_status_line(
                    &Color::Yellow,
                    format!("Waiting for sharing side to come online..."),
                );
                thread::sleep(time::Duration::from_millis(50));
                continue;
            };
        }
        print_status_line(&Color::Green, "Retrieving file information...");

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

        let fname: String = fname.unwrap_or_else(|| format!("{}.file", path));
        let mut dest = File::create(&fname).unwrap();

        let file_size: f64 = result
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or("0")?
            .to_str()?
            .to_string()
            .parse::<f64>()
            .unwrap()
            / 1000000.0;

        print_status_line(
            &Color::Green,
            format!(" {}: {}% of {}mb", &fname, 0, file_size),
        );

        let mut downloaded_megabytes: f64 = 0.0;
        let mut last_write = Instant::now();
        // bytes per second
        let mut speed: f64 = -1.0;
        let mut downloaded_bytes_last_second = 0;
        while let Some(chunk) = result.chunk().await? {
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
                print_status_line(
                    &Color::Green,
                    format!(
                        "{}: Downloaded {:.3}mb {:.3}mb {:.3}mb/s",
                        fname, downloaded_megabytes, file_size, speed
                    ),
                );
            } else {
                let percent = downloaded_megabytes as f32 / file_size as f32 * 100.0;
                print_status_line(
                    &Color::Green,
                    format!(
                        "{}: {:.3}mb of {:.3}mb {:.1}% {:.3}mb/s",
                        fname, downloaded_megabytes, file_size, percent, speed
                    ),
                );
            }
        }
        println!("\n");
        break Ok(());
    }
}
