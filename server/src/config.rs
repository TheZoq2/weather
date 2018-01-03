use std::path::{PathBuf, Path};
use error::Result;

use std::fs::File;
use std::io::prelude::*;

use toml;

#[derive(Deserialize)]
pub struct Config {
    pub http_port: u16,
    pub http_address: String,
    pub tcp_port: u16,
    pub tcp_address: String,
    pub log_filename: PathBuf,
}

pub fn read_config(config_path: &Path) -> Result<Config> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(toml::from_str(&content)?)
}
