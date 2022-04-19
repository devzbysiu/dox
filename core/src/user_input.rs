use crate::cfg::{self, config_path, Config};
use crate::helpers::PathRefExt;
use crate::prompt;
use crate::result::{DoxErr, Result};

use inquire::error::InquireError;
use std::fs::create_dir_all;
use std::path::PathBuf;
use tracing::{debug, instrument};

#[instrument]
pub fn handle_config(path_override: Option<String>) -> Result<Config> {
    let config_path = path_override.map_or(config_path(), PathBuf::from);
    let cfg = if config_path.exists() {
        debug!("loading config from '{}'", config_path.str());
        cfg::load(config_path)?
    } else {
        debug!("config path '{}' doesn't exist", config_path.str());
        let cfg = config_from_user()?;
        cfg::store(config_path, &cfg)?;
        cfg
    };
    prepare_directories(&cfg)?;
    Ok(cfg)
}

fn config_from_user() -> Result<Config> {
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
    check_thumnbails_dir(config)?;
    check_watched_dir(config)?;
    check_index_dir(config)?;
    Ok(())
}

fn check_thumnbails_dir(config: &Config) -> Result<()> {
    if config.thumbnails_dir.exists() && !config.thumbnails_dir.is_dir() {
        return Err(DoxErr::InvalidThumbnailPath(format!(
            "It needs to be a directory: '{}'",
            config.thumbnails_dir.str()
        )));
    }
    create_dir_all(&config.thumbnails_dir)?;
    if config.thumbnails_dir.read_dir()?.next().is_some() {
        return Err(DoxErr::InvalidThumbnailPath(format!(
            "Directory needs to be empty: '{}'",
            config.thumbnails_dir.str()
        )));
    }
    Ok(())
}

fn check_watched_dir(config: &Config) -> Result<()> {
    if config.watched_dir.exists() && !config.watched_dir.is_dir() {
        return Err(DoxErr::InvalidWatchedDirPath(format!(
            "It needs to be a directory: '{}'",
            config.watched_dir.str()
        )));
    }
    create_dir_all(&config.watched_dir)?;
    Ok(())
}

fn check_index_dir(config: &Config) -> Result<()> {
    if config.index_dir.exists() && !config.index_dir.is_dir() {
        return Err(DoxErr::InvalidIndexPath(format!(
            "It needs to be a directory: '{}'",
            config.index_dir.str()
        )));
    }
    Ok(())
}
