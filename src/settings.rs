use config::{Config, ConfigError};
use serde::Deserialize;

// config file directory
const COFIG_DIR: &'static str = "./config/Settings.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub redis: Redis,
    pub code: Code,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub ip: String,
    pub port: u32,
    pub worker: u32,
}

impl Server {
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Redis {
    pub url: String,
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout_secs: u64,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name(COFIG_DIR))
            .build()?;

        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Code {
    pub length: u64,
}
