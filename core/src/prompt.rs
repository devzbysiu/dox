use crate::cfg;
use crate::cfg::Config;
use crate::helpers::PathBufExt;
use crate::result::Result;

use inquire::{required, CustomType, Text};
use std::path::PathBuf;
use std::time::Duration;

pub fn show() -> Result<Config> {
    let config = Config::default();
    let watched_dir = PathBuf::from(
        Text::new("Path to a directory you want to watch for changes:")
            .with_validator(required!())
            .prompt()?,
    );
    let index_dir = PathBuf::from(
        Text::new("Path to a directory for storing index files:")
            .with_default(config.index_dir.str())
            .prompt()?,
    );
    let cooldown_time = Duration::from_secs(
        CustomType::<u64>::new("Cooldown time - # of seconds after which indexing starts:")
            .with_default((config.cooldown_time.as_secs(), &|secs| format!("{}", secs)))
            .prompt()?,
    );

    let config = Config {
        watched_dir,
        index_dir,
        cooldown_time,
    };

    cfg::store(&config)?;
    Ok(config)
}
