use std::fs::File;
use std::io::prelude::*;
use toml;

/// Configuration for Livy Manager
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub http: HTTP,
}

impl Config {
    /// Creates a new `Config` from the file on `conf_path`.
    pub fn from(conf_path: &str) -> Config {
        let mut f = File::open(conf_path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        toml::from_str(contents.as_str()).unwrap()
    }
}

/// Configuration for HTTP
#[derive(Clone, Debug, Deserialize)]
pub struct HTTP {
    pub addr: String,
}
