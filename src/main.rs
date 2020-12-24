#![feature(str_split_once)]


use libtor::{Tor, TorFlag, TorAddress, HiddenServiceVersion, LogDestination, LogLevel};
use std::{thread, time};
use reqwest;
use std::thread::JoinHandle;

use std::io::prelude::*;
use error_chain::error_chain;
use std::fs::File;
use reqwest::header::{CONTENT_LENGTH};
use warp::{Filter, reply::Reply};
use std::path::Path;

use structopt::StructOpt;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use url::{Url, ParseError};

extern crate tempdir;
use tempdir::TempDir;


const SOCKS5_PORT: u16 = 19050;

fn get_hidden_service_hostname(hidden_service_dir: String) -> std::io::Result<String> {
    let file_name = format!("{}/hostname", hidden_service_dir);
    let mut file = File::open(file_name);

    while file.is_err() {
        thread::sleep(time::Duration::from_millis(50));
        file = File::open(format!("{}/hostname", hidden_service_dir));
    }

    let mut file = file.unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents.trim().to_string());
}


fn start_tor_share(dir_tor: &Path, dir_tor_hs: &Path, port: u16) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(dir_tor.to_str().unwrap().into()))
        .flag(TorFlag::HiddenServiceDir(dir_tor_hs.to_str().unwrap().into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(TorAddress::Port(80), Some(TorAddress::AddressPort("127.0.0.1".into(), port).into()).into()))
        //.flag(TorFlag::LogTo(LogLevel::Err, LogDestination::Stderr))
        .start_background();
    return torthread;
}

fn start_tor_download() -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(SOCKS5_PORT))
        //.flag(TorFlag::LogTo(LogLevel::Err, LogDestination::Stderr))
        .start_background();
    return torthread;
}

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
         ParseIntError(std::num::ParseIntError);
         ToStrError(reqwest::header::ToStrError);
     }
}
async fn download_file(hidden_service: String, path: String) -> Result<()> {
    let socks5_url = format!("socks5h://127.0.0.1:{}", SOCKS5_PORT);
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&socks5_url)?)
        .build()?;

    let url = format!("http://{}/{}", hidden_service, path);
    println!("Downloading file from: {}", url);
    loop {
        let result = client.get(&url).send().await;
        if let Err(e) = result {
            if e.is_connect() {
                println!("Waiting for socks5 to be online... {}", e);
                //println!("{:?}", e);
                thread::sleep(time::Duration::from_millis(50));
                continue
            } else {
                eprintln!("Error: {}", e);
                break Ok(())
            }
        }

        let mut result = result.unwrap();
        let fname = if let Some(content_disposition) = result.headers().get("Content-Disposition") {
            let content_disposition: String = content_disposition.to_str().unwrap().into();
            if let Some(filename_index) = content_disposition.rfind("filename=\"") {
                Some(content_disposition.chars().skip(filename_index + 10).take_while(|x| *x != '"').collect())
            } else {
                None
            }
        } else {
            None
        };



        let fname: String = fname.unwrap_or_else(|| format!("{}.file", path));
        println!("Filename: {}", fname);
        let mut dest = File::create(fname).unwrap();

        
        let file_size: usize = result.headers()
            .get(CONTENT_LENGTH)
            .ok_or("0")?.to_str()?.to_string().parse()?;
        println!("File size: {:?}", file_size);

        let mut downloaded_bytes: usize = 0;
        while let Some(chunk) = result.chunk().await? {
            dest.write(&chunk);
            downloaded_bytes = downloaded_bytes + chunk.len();
            if file_size == 0 {
                print!("\rDownloaded bytes: {}", downloaded_bytes);
            } else {
                print!("\rDownloaded: {:.1}%", (downloaded_bytes as f32 / file_size as f32) * 100.0);
            }

        }
        println!("\n");
        break Ok(())
    }
}

fn lossy_file_name(file: &warp::fs::File) -> Option<String> {
    let file_name = file.path()
        .file_name().unwrap_or_default().to_str().unwrap_or_default();
    Some(file_name.into())
}

async fn share_file(path: String, id: String) {
    
    pretty_env_logger::init();

    println!("Serving file {} under /{}", path, id);
    let add_headers = |file: warp::filters::fs::File| {
        let filename = lossy_file_name(&file).unwrap_or_else(|| {
            println!("Couldn't get filename");
            "".into()
        });
        warp::reply::with_header(file, "Content-Disposition", format!("attachment; filename=\"{}\"", filename))
    };
    let examples = warp::path(id).and(warp::fs::file(path)).map(add_headers);

    // GET /{id}... => {file}
    let routes = examples;

    dbg!("Starting http server");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    
}

fn generate_random_id() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    rand_string
}

#[derive(Debug, StructOpt)]
#[structopt(name = "TorShare", about = "A CLI tool to share and download files and folders through tor.")]
enum CliOptions {
    #[structopt(flatten)]
    GeneralOptions(GeneralOptions),


    Download {
        url: String
    },

    Share {
        file_or_folder: String
    }

}

#[derive(Debug, StructOpt)]
struct GeneralOptions {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
}

struct TorShareUrl {
    hostname: String,
    path: String
}
fn parse_torshare_url(url: String) -> Option<TorShareUrl> {

    if let Some((hostname, path)) =  url.split_once('/') {
        if !hostname.ends_with(".onion") {
            None
        } else {
            Some(TorShareUrl { hostname: hostname.into(), path: path.into() })
        }
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    let options: CliOptions = CliOptions::from_args();
    
    match options {
        CliOptions::Download { url } => {
            let tor_share_url = parse_torshare_url(url);

            match tor_share_url {
                Some(tor_share_url) => {
                    let torthread = start_tor_download();
                    dbg!("Ready!");
                    download_file(tor_share_url.hostname, tor_share_url.path).await;
                }
                None => {
                    println!("Invalid url");
                }
            }

        }
        CliOptions::Share { file_or_folder } => {
            let port_webserver: u16 = 8080;
            let tmp_tor_dir = TempDir::new("tor-share").unwrap();
            let tmp_tor_dir_hs = tmp_tor_dir.path().join("hs");
            let torthread = start_tor_share(&tmp_tor_dir.path(), &tmp_tor_dir_hs, port_webserver);
            dbg!("Ready!");
        
            let hidden_service = get_hidden_service_hostname(tmp_tor_dir_hs.to_str().unwrap().into()).unwrap_or("Error".to_string());
            
            let path = generate_random_id();
            let share = share_file(file_or_folder.into(), path.clone());
            println!("Sharing now!\n\ttorshare download {}/{}\n", hidden_service, path);
            share.await;
            dbg!("Stopped sharing file");        
        }
        _ => {}
    }    
}
