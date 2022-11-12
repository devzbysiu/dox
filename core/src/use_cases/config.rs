//! Interface for loading and saving the [`Config`] structure.
//!
//! The actual place where the config will be saved to or read from is not tight to this interface
//! and it's considered to be implementation detail.
use crate::result::ConfigurationErr;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub type CfgResolver = Box<dyn ConfigResolver>;

pub type CfgLoader = Box<dyn ConfigLoader>;

/// Responsible for reading/saving the configuration from/to some medium.
///
/// Used medium is the implementation detail and is not part of this interface.
pub trait ConfigLoader: Send {
    /// Reads the configuration.
    ///
    /// This reads the configuration pointed by `PathBuf`. The `path` argument doesn't need to
    /// represent the location on the File System, this is the implementation detail.
    fn load(&self, path: &Path) -> Result<Config, ConfigurationErr>;

    /// Saves the configuration.
    ///
    /// This saves the configuration in the place pointed by `PathBuf`. It doesn't mean that this
    /// should be saved on the disk, the medium is the detail of the implementation.
    fn store(&self, path: &Path, cfg: &Config) -> Result<(), ConfigurationErr>;
}

/// Handles config override.
///
/// When user specifies configuration path during startup, this interface handles this case.
pub trait ConfigResolver: Send {
    /// Loads the [`Config`] using specified path.
    ///
    /// This method should read the configuration using the path specified as an argument.
    /// If the path is `None`, then no override takes place and configuration should be loaded from
    /// original path.
    fn handle_config(&self, path_override: Option<PathBuf>) -> Result<Config, ConfigurationErr>;
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Config {
    pub watched_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub index_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watched_dir: PathBuf::from(""),
            thumbnails_dir: thumbnails_dir_default(),
            index_dir: index_dir_default(),
        }
    }
}

impl AsRef<Config> for Config {
    fn as_ref(&self) -> &Config {
        self
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_config() {
        // given
        let cfg = Config {
            watched_dir: PathBuf::from(""),
            thumbnails_dir: dirs::data_dir().unwrap().join("dox/thumbnails"),
            index_dir: dirs::data_dir().unwrap().join("dox/index"),
        };

        // when
        let default_cfg = Config::default();

        // then
        assert_eq!(cfg, default_cfg);
    }
}
