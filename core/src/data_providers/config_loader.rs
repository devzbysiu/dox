use crate::configuration::cfg::Config;
use crate::result::{DoxErr, Result};
use crate::use_cases::config::ConfigLoader;

use std::fs::create_dir_all;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::PathBuf;
use tracing::instrument;

pub struct FsConfigLoader;

impl ConfigLoader for FsConfigLoader {
    #[instrument(skip(self))]
    fn load(&self, path: PathBuf) -> Result<Config> {
        Ok(toml::from_str(&read_to_string(path)?)?)
    }

    #[instrument(skip(self))]
    fn store(&self, path: PathBuf, cfg: &Config) -> Result<()> {
        let config_dir = path.parent().ok_or_else(|| {
            DoxErr::InvalidConfigPath("Can't use '/' as a configuration path".into())
        })?;
        create_dir_all(config_dir)?;
        let mut file = File::create(path)?;
        file.write_all(toml::to_string(cfg)?.as_bytes())?;
        Ok(())
    }
}
