use crate::result::{DoxErr, Result};

use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub watched_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub index_dir: PathBuf,
    pub cooldown_time: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watched_dir: PathBuf::from(""),
            thumbnails_dir: thumbnails_dir_default(),
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
        .join("dox/index")
}

fn thumbnails_dir_default() -> PathBuf {
    dirs::data_dir()
        .expect("failed to read system data path")
        .join("dox/thumbnails")
}

pub fn store<P: AsRef<Path>>(path: P, cfg: &Config) -> Result<()> {
    debug!("saving '{:?}' to '{}'", &cfg, path.as_ref().display());
    let config_dir = path
        .as_ref()
        .parent()
        .ok_or_else(|| DoxErr::InvalidConfigPath("Can't use '/' as a configuration path".into()))?;
    create_dir_all(config_dir)?;
    let mut file = File::create(path)?;
    file.write_all(toml::to_string(cfg)?.as_bytes())?;
    Ok(())
}
