use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use libtor::{HiddenServiceVersion, LogDestination, LogLevel, Tor, TorAddress, TorFlag};
use tempdir::TempDir;
use std::fs::File;
use std::io::prelude::*;


use rand::prelude::*;


pub struct TorDirectory {
    pub tor: Rc<String>,
    pub hidden_service: Rc<String>,
    tempdir: Option<TempDir>
}

impl TorDirectory {
    pub fn from_tempdir() -> Rc<Self> {
        let tmp_tor_dir = TempDir::new("tor-share").unwrap();
        let tmp_tor_dir2: String = tmp_tor_dir.path().to_string_lossy().into();
        let tmp_tor_dir_hs = tmp_tor_dir.path().join("hs").to_string_lossy().into();
        Rc::new(TorDirectory {
            tor: Rc::new(tmp_tor_dir2),
            hidden_service: Rc::new(tmp_tor_dir_hs),
            tempdir: Some(tmp_tor_dir)
        })
    }

    pub fn drop_if_temp(&self) {
        if let Some(tempdir) = &self.tempdir {
            drop(tempdir);
        }
    }
}


pub struct TorHiddenServiceConfig {
    pub local_host: Rc<String>,
    pub local_port: u16
}



impl TorHiddenServiceConfig {
    pub fn new(local_host: String, local_port: u16) -> Rc<Self> {
        Rc::new(Self {
            local_host: Rc::new(local_host),
            local_port: local_port
        })
    }
    
    pub fn from_random_port() -> Rc<Self> {
        let rand_port = random_port();
        Self::new(
            "127.0.0.1".into(),
            rand_port,
        )
    }
}

pub fn start_tor_hidden_service(tor_dir: Rc<TorDirectory>, config: Rc<TorHiddenServiceConfig>) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(tor_dir.tor.as_str().into()))
        .flag(TorFlag::SocksPort(0))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::HiddenServiceDir(
            tor_dir.hidden_service.as_str().into(),
        ))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(80),
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

pub fn start_tor_socks5(tor_dir: Rc<TorDirectory>, socks5: Rc<TorSocks5>) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(tor_dir.tor.as_str().into()))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::SocksPort(socks5.port))
        //.flag(TorFlag::LogTo(LogLevel::Err, LogDestination::Stderr))
        //.flag(TorFlag::Quiet())
        .start_background();
    return torthread;
}

pub fn get_hidden_service_hostname(tor_dir: Rc<TorDirectory>) -> std::io::Result<String> {
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
    pub host: Rc<String>,
    pub port: u16
}

impl TorSocks5 {
    pub fn new(host: String, port: u16) -> Rc<Self> {
        Rc::new(TorSocks5 { host: Rc::new(host), port })
    }
    pub fn from_random_port() -> Rc<Self> {
        let rand_port = random_port();
        Self::new("127.0.0.1".into(), rand_port)
    }

    pub fn to_string(&self) -> String {
       format!("socks5h://{}:{}", self.host.clone(), &self.port)
    }
}
