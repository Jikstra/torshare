use std::path::Path;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use libtor::{HiddenServiceVersion, LogDestination, LogLevel, Tor, TorAddress, TorFlag};
use std::fs::File;
use std::io::prelude::*;


use rand::prelude::*;

pub fn start_tor_hidden_service(
    dir_tor: &Path,
    dir_tor_hs: &Path,
    port: u16,
) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory(dir_tor.to_str().unwrap().into()))
        .flag(TorFlag::SocksPort(0))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::HiddenServiceDir(
            dir_tor_hs.to_str().unwrap().into(),
        ))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(80),
            Some(TorAddress::AddressPort("127.0.0.1".into(), port).into()).into(),
        ))
        .flag(TorFlag::LogTo(
            LogLevel::Notice,
            LogDestination::File("/dev/null".into()),
        ))
        .flag(TorFlag::Quiet())
        .start_background();
    return torthread;
}

pub fn start_tor_socks5(socks5_port: u16) -> JoinHandle<std::result::Result<u8, libtor::Error>> {
    let torthread = Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::SocksPort(socks5_port))
        //.flag(TorFlag::LogTo(LogLevel::Err, LogDestination::Stderr))
        .flag(TorFlag::Quiet())
        .start_background();
    return torthread;
}

pub fn get_hidden_service_hostname(hidden_service_dir: String) -> std::io::Result<String> {
    let file_name = format!("{}/hostname", hidden_service_dir);
    let mut file = File::open(file_name);

    while file.is_err() {
        thread::sleep(Duration::from_millis(50));
        file = File::open(format!("{}/hostname", hidden_service_dir));
    }

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
    host: String,
    port: u16
}

impl TorSocks5 {
    pub fn start_background(port: u16) -> Self {
        start_tor_socks5(port);
        Self { host: "127.0.0.1".into(), port}
    }

    pub fn start_background_on_random_port() -> Self {
        let rand_port = random_port();
        //let rand_port = 1997;
        //println!("Port {}\n\n\n\n", rand_port);
        TorSocks5::start_background(rand_port)
    }

    pub fn to_string(&self) -> String {
       format!("socks5h://{}:{}", &self.host, &self.port)
    }
}