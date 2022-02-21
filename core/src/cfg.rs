use crate::result::Result;

use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub watched_dir: PathBuf,
    pub index_dir: PathBuf,
    pub cooldown_time: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watched_dir: PathBuf::from(""),
            index_dir: index_dir_default(),
            cooldown_time: Duration::from_secs(60),
        }
    }
}

pub fn read_config<P: AsRef<Path>>(config_path: P) -> Result<Config> {
    debug!("reading config from {}...", config_path.as_ref().display());
    Ok(toml::from_str(&read_to_string(config_path)?)?)
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("failed to read system config direcory")
        .join("dox/dox.toml")
}

fn index_dir_default() -> PathBuf {
    dirs::data_dir()
        .expect("failed to read system data path")
        .join("dox")
}

pub fn store(config: &Config) -> Result<()> {
    let mut file = File::create(config_path())?;
    file.write_all(toml::to_string(config)?.as_bytes())?;
    Ok(())
}
