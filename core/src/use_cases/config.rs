//! Interface for loading and saving the [`Config`] structure.
//!
//! The actual place where the config will be saved to or read from is not tight to this interface
//! and it's considered to be implementation detail.
use crate::entities::file::{Filename, Thumbnailname};
use crate::entities::user::User;
use crate::result::ConfigurationErr;

use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
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
    pub docs_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub index_dir: PathBuf,
}

impl Config {
    pub fn thumbnail_path(&self, user: &User, name: &Thumbnailname) -> PathBuf {
        self.thumbnails_dir.join(relative_path(user, name))
    }

    pub fn document_path(&self, user: &User, name: &Filename) -> PathBuf {
        self.docs_dir.join(relative_path(user, name))
    }

    pub fn watched_path(&self, user: &User, name: &Filename) -> PathBuf {
        self.watched_dir.join(relative_path(user, name))
    }
}

fn relative_path<D: Display>(user: &User, filename: &D) -> String {
    format!("{}/{}", b64.encode(&user.email), filename)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watched_dir: PathBuf::from(""),
            docs_dir: docs_dir_default(),
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

fn docs_dir_default() -> PathBuf {
    dirs::data_dir()
        .expect("failed to read system data path")
        .join("dox/docs")
}

fn thumbnails_dir_default() -> PathBuf {
    dirs::data_dir()
        .expect("failed to read system data path")
        .join("dox/thumbnails")
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        // given
        let cfg = Config {
            watched_dir: PathBuf::from(""),
            docs_dir: dirs::data_dir().unwrap().join("dox/docs"),
            thumbnails_dir: dirs::data_dir().unwrap().join("dox/thumbnails"),
            index_dir: dirs::data_dir().unwrap().join("dox/index"),
        };

        // when
        let default_cfg = Config::default();

        // then
        assert_eq!(cfg, default_cfg);
    }

    #[test]
    fn thumbnail_path_returns_correct_joined_path() -> Result<()> {
        // given
        let thumbnails_dir = tempdir()?;
        let config = Config {
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            ..Default::default()
        };
        let user: User = Faker.fake();
        let thumbnailname: Thumbnailname = Faker.fake();
        let relative_path = format!("{}/{}", b64.encode(&user.email), &thumbnailname);

        // when
        let thumbnail_path = config.thumbnail_path(&user, &thumbnailname);

        // then
        assert_eq!(thumbnail_path, thumbnails_dir.path().join(relative_path));

        Ok(())
    }

    #[test]
    fn document_path_returns_correct_joined_path() -> Result<()> {
        // given
        let documents_dir = tempdir()?;
        let config = Config {
            docs_dir: documents_dir.path().to_path_buf(),
            ..Default::default()
        };
        let user: User = Faker.fake();
        let filename: Filename = Faker.fake();
        let relative_path = format!("{}/{}", b64.encode(&user.email), &filename);

        // when
        let document_path = config.document_path(&user, &filename);

        // then
        assert_eq!(document_path, documents_dir.path().join(relative_path));

        Ok(())
    }

    #[test]
    fn watched_path_returns_correct_joined_path() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let config = Config {
            watched_dir: watched_dir.path().to_path_buf(),
            ..Default::default()
        };
        let user: User = Faker.fake();
        let filename: Filename = Faker.fake();
        let relative_path = format!("{}/{}", b64.encode(&user.email), &filename);

        // when
        let watched_path = config.watched_path(&user, &filename);

        // then
        assert_eq!(watched_path, watched_dir.path().join(relative_path));

        Ok(())
    }
}
