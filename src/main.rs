#![feature(str_split_once)]

extern crate tempdir;

mod cli;
use cli::{CliOptions, Color, print_status_line, save_cursor_position};
use structopt::StructOpt;

mod tor_utils;
use tor_utils::{TorDirectory, TorHiddenServiceConfig, TorSocks5, get_hidden_service_hostname, start_tor_hidden_service};

mod tor_share_url;

mod share;
use share::{ShareOptions, ShareState, share_file};

mod download_file;
use download_file::{DownloadOptions, DownloadState, download_file};

async fn download(download_options: &DownloadOptions) {

    //dbg!("Ready!");
    download_file(&download_options, |download_state| {
        save_cursor_position();
        match download_state {
            DownloadState::ConnectingWaitingForTor => {
                print_status_line(&Color::Yellow, "Connecting to tor network...");
            },
            DownloadState::ConnectingWaitingForProxy(tor_socks5) => {
                print_status_line(
                    &Color::Yellow,
                    format!("Connecting to tor network... Waiting for proxy... {}", tor_socks5.port),
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

async fn share(share_options: &ShareOptions) {
    save_cursor_position();
    share_file(&share_options, |share_state| {
        match share_state {
            ShareState::ConnectingStartingTor => {
                print_status_line(&Color::Yellow, "Starting Tor");

            },
            ShareState::OnlineSharingNow(tor_share_url) => {
                print_status_line(
                    &Color::Green,
                    format!(
                        "Sharing now! Run following command to download: \"torshare download {}\"",
                        tor_share_url.to_string()
                    ),
                );
            },
            ShareState::OfflineStopped => {
                print_status_line(&Color::Red, "Stopped sharing\n");

            },
            ShareState::OfflineError(err) => {
                print_status_line(&Color::Red, format!("Error: {}\n", err));

            }
        }
    }).await;
}

#[tokio::main]
async fn main() {
    let options: CliOptions = CliOptions::from_args();

    match &options {
        CliOptions::Download { download_options} => {
            download(&download_options).await;
        }
        CliOptions::Share { share_options } => {
            share(&share_options).await;
        }
    }


}
