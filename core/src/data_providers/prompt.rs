//! This module allows to gather configuration data from the user. It displays interactive prompt
//! in the terminal which asks the user for the data.
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::config::Config;

use inquire::{required, CustomType, CustomUserError, Text};
use std::fs;
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Shows prompt in the terminal.
///
/// The prompt will get all the fields needed in the [`Config`] struct. It uses [`inquire`] to
/// render the prompt.
pub fn show() -> Result<Config> {
    let config = Config::default();
    Ok(Config {
        watched_dir: watched_dir_prompt()?,
        thumbnails_dir: thumbnails_dir_prompt(&config)?,
        index_dir: index_dir_prompt(&config)?,
        cooldown_time: cooldown_time_prompt(&config)?,
        notifications_addr: notifications_addr_prompt()?,
        websocket_cleanup_time: websocket_cleanup_time_prompt(&config)?,
    })
}

fn watched_dir_prompt() -> Result<PathBuf> {
    Ok(PathBuf::from(
        Text::new("Path to a directory you want to watch for changes:")
            .with_suggester(&path_suggester)
            .with_validator(required!())
            .prompt()?,
    ))
}

fn path_suggester(input: &str) -> std::result::Result<Vec<String>, CustomUserError> {
    let path = Path::new(input);
    if path.is_dir() {
        return Ok(vec![input.to_string()]);
    }
    let parent = path.parent();
    if parent.is_none() {
        return Ok(vec![]);
    }
    let dir = fs::read_dir(parent.unwrap()); // can unwrap, it's checked earlier
    Ok(dir
        .unwrap()
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.as_path().is_dir())
        .map(|path| path.as_path().string())
        .filter(|path| path.contains(input))
        .collect())
}

fn thumbnails_dir_prompt(config: &Config) -> Result<PathBuf> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing thumbnails:")
            .with_suggester(&path_suggester)
            .with_default(config.thumbnails_dir.str())
            .prompt()?,
    ))
}

fn index_dir_prompt(config: &Config) -> Result<PathBuf> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing index files:")
            .with_suggester(&path_suggester)
            .with_default(config.index_dir.str())
            .prompt()?,
    ))
}

fn cooldown_time_prompt(config: &Config) -> Result<Duration> {
    Ok(Duration::from_secs(
        CustomType::<u64>::new("Cooldown time - # of seconds after which indexing starts:")
            .with_default((config.cooldown_time.as_secs(), &|secs| format!("{}", secs)))
            .prompt()?,
    ))
}

fn notifications_addr_prompt() -> Result<SocketAddrV4> {
    Ok(Text::new("IP address of notifications:")
        .with_default("0.0.0.0:8001")
        .prompt()?
        .parse()?)
}

fn websocket_cleanup_time_prompt(config: &Config) -> Result<Duration> {
    Ok(Duration::from_secs(
        CustomType::<u64>::new(
            "Websocket cleanup - # of seconds after which websockets are checked and cleaned up:",
        )
        .with_default((config.cooldown_time.as_secs(), &|secs| format!("{}", secs)))
        .prompt()?,
    ))
}
