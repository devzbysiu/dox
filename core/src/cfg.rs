use crate::error::Result;
use log::debug;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub watched_dir: PathBuf,
    pub index_dir: PathBuf,
    pub cooldown_time: Duration,
}

pub fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Config> {
    debug!("reading config from {}...", config_path.as_ref().display());
    Ok(toml::from_str(&read_to_string(config_path)?)?)
}
