use crate::cfg;
use crate::cfg::Config;
use crate::helpers::PathBufExt;
use crate::result::Result;

use inquire::{required, CustomType, Text};
use std::path::PathBuf;
use std::time::Duration;

pub fn show() -> Result<Config> {
    let config = Config::default();
    cfg::store(&Config {
        watched_dir: watched_dir_prompt()?,
        index_dir: index_dir_prompt(&config)?,
        cooldown_time: cooldown_time_prompt(&config)?,
    })?;
    Ok(config)
}

fn watched_dir_prompt() -> Result<PathBuf> {
    Ok(PathBuf::from(
        Text::new("Path to a directory you want to watch for changes:")
            .with_validator(required!())
            .prompt()?,
    ))
}

fn index_dir_prompt(config: &Config) -> Result<PathBuf> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing index files:")
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
