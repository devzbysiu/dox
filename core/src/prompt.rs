use crate::cfg::Config;
use crate::helpers::{PathBufExt, PathExt};
use crate::result::Result;

use inquire::{required, CustomType, Text};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn show() -> Result<Config> {
    let config = Config::default();
    Ok(Config {
        watched_dir: watched_dir_prompt()?,
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
    // TODO: think if this can be simplified
    let input = if input.is_empty() { "/" } else { input };
    let mut dir = fs::read_dir(input);
    if dir.is_err() {
        let parent = Path::new(input).parent();
        if parent.is_none() {
            return vec![];
        }
        dir = fs::read_dir(parent.unwrap()); // can unwrap, it's checked earlier
        if dir.is_err() {
            return vec![];
        }
    }
    dir.unwrap() // can unwrap because it's checked above
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.as_path().is_dir())
        .map(|path| path.as_path().string())
        .filter(|path| path.contains(input))
        .collect::<Vec<String>>()
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
