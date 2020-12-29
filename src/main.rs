#![feature(str_split_once)]

extern crate tempdir;
use tempdir::TempDir;

use async_ctrlc::CtrlC;
use futures_lite::future::FutureExt;

mod cli;
use cli::{print_status_line, save_cursor_position, CliOptions, Color};
use structopt::StructOpt;

mod tor_utils;
use tor_utils::{TorSocks5, get_hidden_service_hostname, start_tor_hidden_service};

mod tor_share_url;
use tor_share_url::TorShareUrl;

mod share;
use share::share_file;

mod download_file;
use download_file::{DownloadState, download_file};

const SOCKS5_PORT: u16 = 1997;

async fn download(url: Option<TorShareUrl>) {
    if url.is_none() {
        print_status_line(&Color::Red, "Error: Not a valid torshare URL\n");
        return
    }

    let tor_share_url = url.unwrap();
    let tor_socks5 = TorSocks5::start_background_on_random_port();

    //dbg!("Ready!");
    download_file(tor_socks5, tor_share_url, |download_state| {
        save_cursor_position();
        match download_state {
            DownloadState::ConnectingWaitingForTor => {
                print_status_line(&Color::Yellow, "Connecting to tor network...");
            },
            DownloadState::ConnectingWaitingForProxy => {
                print_status_line(
                    &Color::Yellow,
                    "Connecting to tor network... Waiting for proxy...",
                );
            }
            DownloadState::ConnectedWaitingForPeer => {
                print_status_line(
                    &Color::Yellow,
                    format!("Waiting for sharing side to come online..."),
                );
            }
            DownloadState::ConnectedRetrievingFileInformation => {
                print_status_line(&Color::Green, "Retrieving file information...");
            }
            DownloadState::ConnectedRetrievedFileInformation ( file_information ) => {
                print_status_line(
                    &Color::Green,
                    format!(" {}: {}% of {}mb", file_information.name, 0, file_information.size),
                );
            }
            DownloadState::ConnectedDownloading ( file_information, download_progress ) => {
                if download_progress.percent == -1.0 {
                    print_status_line(
                        &Color::Green,
                        format!(
                            "{}: {:.3}mb of unknown {:.3}mb/s",
                            file_information.name, download_progress.downloaded_megabytes, download_progress.speed
                        ),
                    );
                    return
                }
                print_status_line(
                    &Color::Green,
                    format!(
                        "{}: {:.3}mb of {:.3}mb {:.1}% {:.3}mb/s",
                        file_information.name, download_progress.downloaded_megabytes, file_information.size, download_progress.percent, download_progress.speed
                    ),
                );
            }
            DownloadState::DisconnectedError ( error ) => {
                print_status_line(
                    &Color::Red,
                    format!(
                        "Error: {}",
                        error
                    ),
                );

            }
        };
    }).await;
}

#[tokio::main]
async fn main() {
    let options: CliOptions = CliOptions::from_args();

    match options {
        CliOptions::Download { url } => {
            download(TorShareUrl::from_str(url)).await;
        }
        CliOptions::Share { file_or_folder } => {
            save_cursor_position();
            print_status_line(&Color::Yellow, "Starting Tor");
            let port_webserver: u16 = 8080;
            let tmp_tor_dir = TempDir::new("tor-share").unwrap();
            let tmp_tor_dir_hs = tmp_tor_dir.path().join("hs");
            let _torthread =
                start_tor_hidden_service(&tmp_tor_dir.path(), &tmp_tor_dir_hs, port_webserver);
            //dbg!("Ready!");
            //print!("[DEBUG] Tempdir: {}", tmp_tor_dir.path().to_str().unwrap());

            let hidden_service_hostname =
                get_hidden_service_hostname(tmp_tor_dir_hs.to_str().unwrap().into())
                    .unwrap_or("Error".to_string());

            let tor_share_url = TorShareUrl::random_path(hidden_service_hostname);
            let share = share_file(file_or_folder.into(), tor_share_url.path.clone());

            let ctrlc = CtrlC::new().expect("cannot create Ctrl+C handler?");
            print_status_line(
                &Color::Green,
                format!(
                    "Sharing now! Run following command to download: \"torshare download {}\"",
                    tor_share_url.to_string()
                ),
            );
            ctrlc.race(share).await;
            drop(tmp_tor_dir);
            //tmp_tor_dir.close().unwrap();
            print_status_line(&Color::Red, "Stopped sharing\n");
        }
        _ => {}
    }
}
