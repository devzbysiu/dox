use crate::cfg::{self, config_path, Config};
use crate::helpers::PathBufExt;
use crate::prompt;
use crate::result::{DoxErr, Result};

use inquire::error::InquireError;
use log::debug;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub fn handle_config(path_override: Option<String>) -> Result<Config> {
    debug!("handling config with {:?}", path_override);
    let config_path = path_override.map_or(config_path(), PathBuf::from);
    if config_path.exists() {
        debug!("loading config from '{}'", config_path.str());
        cfg::read_config(config_path)
    } else {
        debug!("config path '{}' doesn't exist", config_path.str());
        let cfg = config_from_user()?;
        prepare_directories(&cfg)?;
        cfg::store(config_path, &cfg)?;
        Ok(cfg)
    }
}

pub fn config_from_user() -> Result<Config> {
    match prompt::show() {
        Ok(cfg) => Ok(cfg),
        Err(DoxErr::Prompt(InquireError::OperationCanceled)) => exit_process(),
        Err(e) => panic!("failed while showing prompt: {}", e),
    }
}

fn exit_process() -> ! {
    debug!("prompt cancelled, exiting process");
    std::process::exit(0);
}

fn prepare_directories(config: &Config) -> Result<()> {
    if config.thumbnails_dir.exists() && !config.thumbnails_dir.is_dir() {
        return Err(DoxErr::InvalidThumbnailPath("It needs to be a directory"));
    }
    create_dir_all(&config.thumbnails_dir)?;
    if config.thumbnails_dir.read_dir()?.next().is_some() {
        return Err(DoxErr::InvalidThumbnailPath("Directory needs to be empty"));
    }
    if config.watched_dir.exists() && !config.watched_dir.is_dir() {
        return Err(DoxErr::InvalidWatchedDirPath("It needs to be a directory"));
    }
    create_dir_all(&config.watched_dir)?;
    if config.index_dir.exists() {
        return Err(DoxErr::InvalidIndexPath("The path is already taken"));
    }
    Ok(())
}
