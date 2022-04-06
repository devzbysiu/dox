use crate::cfg::Config;
use crate::helpers::PathRefExt;
use crate::result::Result;

use inquire::{required, CustomType, Text};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn show() -> Result<Config> {
    let config = Config::default();
    Ok(Config {
        watched_dir: watched_dir_prompt()?,
        thumbnails_dir: thumbnails_dir_prompt(&config)?,
        index_dir: index_dir_prompt(&config)?,
        cooldown_time: cooldown_time_prompt(&config)?,
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

fn path_suggester(input: &str) -> Vec<String> {
    let path = Path::new(input);
    if path.is_dir() {
        return vec![input.to_string()];
    }
    let parent = path.parent();
    if parent.is_none() {
        return vec![];
    }
    let dir = fs::read_dir(parent.unwrap()); // can unwrap, it's checked earlier
    dir.unwrap()
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.as_path().is_dir())
        .map(|path| path.as_path().string())
        .filter(|path| path.contains(input))
        .collect()
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
