use std::rc::Rc;

use async_ctrlc::CtrlC;
use futures::Future;
use warp::Filter;
use crate::{print_status_line, tor_share_url::TorShareUrl, tor_utils::{TorDirectory, get_hidden_service_hostname, start_tor_hidden_service}};
use crate::Color;

use futures_lite::future::FutureExt;


use crate::tor_utils::TorHiddenServiceConfig;


pub enum ShareState {
    ConnectingStartingTor,
    OnlineSharingNow(TorShareUrl),
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

pub async fn share_file(tor_dir: Rc<TorDirectory>, hidden_service_config: Rc<TorHiddenServiceConfig>, file_or_folder: String,  cb: impl Fn(ShareState)) {
    cb(ShareState::ConnectingStartingTor);

    let _torthread =
        start_tor_hidden_service(Rc::clone(&tor_dir), Rc::clone(&hidden_service_config));

    let hidden_service_hostname =
        get_hidden_service_hostname(Rc::clone(&tor_dir))
            .unwrap_or("Error".to_string());

    let tor_share_url = TorShareUrl::random_path(hidden_service_hostname);
    let share = start_webserver(Rc::clone(&hidden_service_config), file_or_folder.into(), tor_share_url.path.clone());

    let ctrlc = CtrlC::new().expect("cannot create Ctrl+C handler?");
    cb(ShareState::OnlineSharingNow(tor_share_url));

    ctrlc.race(share).await;
    tor_dir.drop_if_temp();

    cb(ShareState::OfflineStopped);
}

fn start_webserver(tor_hidden_service_config: Rc<TorHiddenServiceConfig>, path: String, id: String) -> impl Future<Output = ()> {
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
