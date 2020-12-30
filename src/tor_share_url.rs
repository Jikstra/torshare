use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::share::TorShareUrlOptions;

#[derive(Debug)]
pub struct TorShareUrl {
    pub hostname: String,
    pub path: String,
}

impl TorShareUrl {
    pub fn from_str(url: &str) -> Result<Self, &str> {
        if let Some((hostname, path)) = url.split_once('/') {
            if !hostname.ends_with(".onion") {
                Err("Hostname doesn't end with .onion")
            } else {
                Ok(TorShareUrl {
                    hostname: hostname.into(),
                    path: path.into(),
                })
            }
        } else {
            Err("Hostname doesn't contain a / delimiter")
        }
    }

    pub fn random_path(hostname: String) -> Self {
        let rand_path: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        TorShareUrl {
            hostname: hostname.into(),
            path: rand_path,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}/{}", &self.hostname, &self.path)
    }

    pub fn to_url(&self) -> String {
        format!("http://{}/{}", &self.hostname, &self.path)
    }
}
