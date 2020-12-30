use std::{num::ParseIntError, str::FromStr};

use async_ctrlc::CtrlC;
use futures::Future;
use warp::Filter;
use crate::{tor_share_url::TorShareUrl, tor_utils::{TorDirOptions, TorDirectory, get_hidden_service_hostname, start_tor_hidden_service}};

use futures_lite::future::FutureExt;
use structopt::StructOpt;

use crate::tor_utils::TorHiddenServiceConfig;
#[derive(Debug, StructOpt)]
pub struct TorShareUrlOptions {
    #[structopt(long)]
    pub path: Option<String>
}

impl TorShareUrlOptions {
    pub fn into_tor_share_url(&self, hostname: &str) -> TorShareUrl {
        if let Some(path) = &self.path {
            TorShareUrl {
                hostname: hostname.clone().into(),
                path: path.clone().into(),
            }
        } else {
            TorShareUrl::random_path(hostname.clone().into())
        }
    }
}

impl FromStr for TorShareUrlOptions {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TorShareUrlOptions { path: Some(s.clone().into())})
    }
}

#[derive(Debug, StructOpt)]
pub struct ShareOptions {
    #[structopt(flatten)]
    pub tor_dir_options: TorDirOptions,
    
    pub file_or_folder: String,
    pub id: Option<String>,

    #[structopt(flatten)]
    pub tor_share_url_options: TorShareUrlOptions,
}

pub enum ShareState<'a> {
    ConnectingStartingTor,
    OnlineSharingNow(&'a TorShareUrl),
    OfflineStopped,
    OfflineError(String)
}

pub fn lossy_file_name(file: &warp::fs::File) -> Option<String> {
    let file_name = file
        .path()
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    Some(file_name.into())
}

pub async fn share_file(share_options: &ShareOptions,  cb: impl Fn(ShareState)) {
    let tor_dir = TorDirectory::from_general_options(&share_options.tor_dir_options); 
    let hidden_service_config = TorHiddenServiceConfig::from_random_port();  
    let _torthread = start_tor_hidden_service(&tor_dir, &hidden_service_config);

    let hidden_service_hostname =
        get_hidden_service_hostname(&tor_dir)
            .unwrap_or("Error".to_string());
    
    let tor_share_url = share_options.tor_share_url_options.into_tor_share_url(&hidden_service_hostname);

    cb(ShareState::ConnectingStartingTor);

    let share = start_webserver(&hidden_service_config, share_options.file_or_folder.clone(), tor_share_url.path.clone());

    let ctrlc = CtrlC::new().expect("cannot create Ctrl+C handler?");
    cb(ShareState::OnlineSharingNow(&tor_share_url));

    ctrlc.race(share).await;
    tor_dir.drop_if_temp();

    cb(ShareState::OfflineStopped);
}

fn start_webserver(tor_hidden_service_config: &TorHiddenServiceConfig, path: String, id: String) -> impl Future<Output = ()> {
    pretty_env_logger::init();

    //println!("Serving file {} under /{}", path, id);
    let add_headers = |file: warp::filters::fs::File| {
        let filename = lossy_file_name(&file).unwrap_or_else(|| {
            println!("Couldn't get filename");
            "".into()
        });
        warp::reply::with_header(
            file,
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
    };
    let examples = warp::path(id).and(warp::fs::file(path)).map(add_headers);

    // GET /{id}... => {file}
    let routes = examples;

    println!("Starting http server on port {}", tor_hidden_service_config.local_port);

    warp::serve(routes).run(([127, 0, 0, 1], tor_hidden_service_config.local_port))
}
