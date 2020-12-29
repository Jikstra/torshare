#![feature(str_split_once)]

extern crate tempdir;
use tempdir::TempDir;

use async_ctrlc::CtrlC;
use futures_lite::future::FutureExt;

mod cli;
use cli::{print_status_line, save_cursor_position, CliOptions, Color};
use structopt::StructOpt;

mod tor_utils;
use tor_utils::{get_hidden_service_hostname, start_tor_hidden_service, start_tor_socks5};

mod tor_share_url;
use tor_share_url::TorShareUrl;

mod share;
use share::share_file;

mod download_file;
use download_file::download_file;

const SOCKS5_PORT: u16 = 1996;

#[tokio::main]
async fn main() {
    let options: CliOptions = CliOptions::from_args();

    match options {
        CliOptions::Download { url } => {
            let tor_share_url = TorShareUrl::from_str(url);

            match tor_share_url {
                Some(tor_share_url) => {
                    let _torthread = start_tor_socks5(SOCKS5_PORT);
                    //dbg!("Ready!");
                    download_file(tor_share_url.hostname, tor_share_url.path).await;
                }
                None => {
                    print_status_line(&Color::Red, "Invalid URL\n");
                }
            }
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
