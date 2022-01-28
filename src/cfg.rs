use anyhow::Result;
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
    Ok(toml::from_str(&read_to_string(
        config_path.as_ref().join("dox.toml"),
    )?)?)
}
