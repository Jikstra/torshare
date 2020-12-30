use std::{path::Path, thread};
use std::thread::JoinHandle;
use std::time::Duration;

use libtor::{HiddenServiceVersion, LogDestination, LogLevel, Tor, TorAddress, TorFlag};
use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;


use rand::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct TorDirOptions {
    #[structopt(long, env = "TOR_DIR")]
    pub tor_dir: Option<String>,
    #[structopt(long, env = "TOR_DIR_HS")]
    pub tor_dir_hs: Option<String>,
}




pub struct TorDirectory {
    pub tor: String,
    pub hidden_service: String,
    tempdir: Option<TempDir>
}

impl TorDirectory {
    pub fn from_tempdir() -> Self {
        let tmp_tor_dir = TempDir::new("tor-share").unwrap();
        let tmp_tor_dir2: String = tmp_tor_dir.path().to_string_lossy().into();
        let tmp_tor_dir_hs = tmp_tor_dir.path().join("hs").to_string_lossy().into();
        TorDirectory {
            tor: tmp_tor_dir2,
            hidden_service: tmp_tor_dir_hs,
            tempdir: Some(tmp_tor_dir)
        }
    }

    pub fn from_general_options(tor_dir_options: &TorDirOptions) ->  Self{
        if  tor_dir_options.tor_dir.is_some() {
            let tor_dir = tor_dir_options.tor_dir.clone().unwrap();
            let hidden_service = if tor_dir_options.tor_dir_hs.is_some() {
                tor_dir_options.tor_dir_hs.clone().unwrap()
            } else {
                let hidden_service = tor_dir.clone();
                let path = Path::new(&hidden_service);
                let path = path.join("hs");
                path.to_string_lossy().into()
                
            };
            return TorDirectory {
                tor: tor_dir.clone(),
                hidden_service,
                tempdir: None
            };
        }
        Self::from_tempdir()

    }

    pub fn drop_if_temp(&self) {
        if let Some(tempdir) = &self.tempdir {
            drop(tempdir);
        }
    }
}


pub struct TorHiddenServiceConfig {
    pub local_host: String,
    pub local_port: u16,
    pub remote_port: u16,
}

impl TorHiddenServiceConfig { 
    pub fn from_random_port() -> Self {
        let rand_port = random_port();
        Self { local_host: "127.0.0.1".into(), local_port: rand_port, remote_port: 80 }
    }
}

pub fn start_tor_hidden_service(tor_dir: &TorDirectory, config: &TorHiddenServiceConfig) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(tor_dir.tor.as_str().into()))
        .flag(TorFlag::SocksPort(0))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::HiddenServiceDir(
            tor_dir.hidden_service.as_str().into(),
        ))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(config.remote_port),
            Some(TorAddress::AddressPort(config.local_host.as_str().into(), config.local_port).into()).into(),
        ))
        .flag(TorFlag::LogTo(
            LogLevel::Notice,
            LogDestination::File("/dev/null".into()),
        ))
        //.flag(TorFlag::Quiet())
        .start_background();
    return torthread;
}

pub fn start_tor_socks5(tor_dir: &TorDirectory, socks5: &TorSocks5) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(tor_dir.tor.as_str().into()))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::SocksPort(socks5.port))
        //.flag(TorFlag::LogTo(LogLevel::Err, LogDestination::Stderr))
        //.flag(TorFlag::Quiet())
        .start_background();
    return torthread;
}

pub fn get_hidden_service_hostname(tor_dir: &TorDirectory) -> std::io::Result<String> {
    let file_name = format!("{}/hostname", tor_dir.hidden_service.clone());

    let file = loop {
        let file = File::open(&file_name);
        if file.is_err() {
            thread::sleep(Duration::from_millis(50));
            continue
        } 
        break file
    };

    

    let mut file = file.unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents.trim().to_string());
}


// ToDo: This is currently a very dumb approach. But should work in most cases and can
// easily get fixed by running torshare again.
pub fn random_port() -> u16 {
    rand::thread_rng().gen_range(1024..65535)
}
pub struct TorSocks5 {
    pub host: String,
    pub port: u16
}

impl TorSocks5 {
    pub fn from_random_port() -> Self {
        let rand_port = random_port();
        Self { host: "127.0.0.1".into(), port: rand_port }
    }

    pub fn to_string(&self) -> String {
       format!("socks5h://{}:{}", self.host.clone(), &self.port)
    }
}
