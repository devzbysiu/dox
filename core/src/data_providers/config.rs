use crate::data_providers::prompt;
use crate::helpers::PathRefExt;
use crate::result::{DoxErr, Result};
use crate::use_cases::config::{Config, ConfigLoader, ConfigResolver};

use inquire::error::InquireError;
use std::fs::create_dir_all;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

pub struct FsConfigLoader;

/// Loads configuration file.
///
/// It reads a toml file from the filesystem and decodes it into [`Config`] structure.
impl ConfigLoader for FsConfigLoader {
    #[instrument(skip(self))]
    fn load(&self, path: &Path) -> Result<Config> {
        Ok(toml::from_str(&read_to_string(path)?)?)
    }

    #[instrument(skip(self))]
    fn store(&self, path: &Path, cfg: &Config) -> Result<()> {
        let config_dir = path.parent().ok_or_else(|| {
            DoxErr::InvalidConfigPath("Can't use '/' as a configuration path".into())
        })?;
        create_dir_all(config_dir)?;
        let mut file = File::create(path)?;
        file.write_all(toml::to_string(cfg)?.as_bytes())?;
        Ok(())
    }
}

/// Handles configuration override and accepts configuration from the user.
///
/// When no configuration exists and no configuration override is passed, the resolver accepts
/// configuration from the user using prompt in the terminal. So the priority order is as follows:
/// 1. Default configuration path. See [`config_path`].
/// 2. Config override.
/// 3. Config from the user.
pub struct FsConfigResolver {
    config_loader: Box<dyn ConfigLoader>,
}

impl FsConfigResolver {
    pub fn new(config_loader: Box<dyn ConfigLoader>) -> Self {
        Self { config_loader }
    }
}

impl ConfigResolver for FsConfigResolver {
    #[instrument(skip(self))]
    fn handle_config(&self, path_override: Option<String>) -> Result<Config> {
        let config_path = path_override.map_or(config_path(), PathBuf::from);
        let cfg = if config_path.exists() {
            debug!("loading config from '{}'", config_path.str());
            self.config_loader.load(&config_path)?
        } else {
            debug!("config path '{}' doesn't exist", config_path.str());
            let cfg = config_from_user()?;
            self.config_loader.store(&config_path, &cfg)?;
            cfg
        };
        prepare_directories(&cfg)?;
        Ok(cfg)
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("failed to read system config direcory")
        .join("dox/dox.toml")
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::data_providers::config::config_path;

    use anyhow::Result;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_load_config() -> Result<()> {
        // given
        let tmp_cfg = tempdir()?;
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
            watched_dir = "/home/zbyniu/Tests/notify"
            thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
            index_dir = "/home/zbyniu/.local/share/dox/index"
            notifications_addr = "0.0.0.0:8001"

            [cooldown_time]
            secs = 1
            nanos = 0

            [websocket_cleanup_time]
            secs = 10
            nanos = 0
            "#,
        )?;
        let expected = Config {
            watched_dir: PathBuf::from("/home/zbyniu/Tests/notify"),
            thumbnails_dir: PathBuf::from("/home/zbyniu/.local/share/dox/thumbnails"),
            index_dir: PathBuf::from("/home/zbyniu/.local/share/dox/index"),
            cooldown_time: Duration::from_secs(1),
            notifications_addr: "0.0.0.0:8001".parse()?,
            websocket_cleanup_time: Duration::from_secs(10),
        };
        let loader = FsConfigLoader;

        // when
        let read_cfg = loader.load(&cfg_path)?;

        // then
        assert_eq!(expected, read_cfg);

        Ok(())
    }

    fn create_config<S: Into<String>, A: AsRef<Path>>(path: A, content: S) -> Result<()> {
        let path = path.as_ref();
        let mut cfg_file = File::create(&path)?;
        cfg_file.write_all(content.into().as_bytes())?;
        Ok(())
    }

    #[test]
    #[should_panic(expected = "missing field `watched_dir`")]
    fn test_load_config_when_missing_watched_dir() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                notifications_addr = "0.0.0.0:8001"

                [cooldown_time]
                secs = 1
                nanos = 0

                [websocket_cleanup_time]
                secs = 10
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `thumbnails_dir`")]
    fn test_load_config_when_missing_thumbnails_dir() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                notifications_addr = "0.0.0.0:8001"

                [cooldown_time]
                secs = 1
                nanos = 0

                [websocket_cleanup_time]
                secs = 10
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `index_dir`")]
    fn test_load_config_when_missing_index_dir() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                notifications_addr = "0.0.0.0:8001"

                [cooldown_time]
                secs = 1
                nanos = 0

                [websocket_cleanup_time]
                secs = 10
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `notifications_addr`")]
    fn test_load_config_when_missing_notifications_addr() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                index_dir = "/home/zbyniu/.local/share/dox/index"

                [cooldown_time]
                secs = 1
                nanos = 0

                [websocket_cleanup_time]
                secs = 10
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `cooldown_time`")]
    fn test_load_config_when_missing_cooldown_time() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                notifications_addr = "0.0.0.0:8001"

                [websocket_cleanup_time]
                secs = 10
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `websocket_cleanup_time`")]
    fn test_load_config_when_missing_websocket_cleanup_time() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                notifications_addr = "0.0.0.0:8001"

                [cooldown_time]
                secs = 1
                nanos = 0
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    fn test_config_path() {
        // given
        let path = dirs::config_dir().unwrap().join("dox/dox.toml");

        // when
        let cfg_path = config_path();

        // then
        assert_eq!(cfg_path, path);
    }

    #[test]
    fn test_store_config() -> Result<()> {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        let cfg = Config {
            watched_dir: PathBuf::from("/watched_dir"),
            thumbnails_dir: PathBuf::from("/thumbnails_dir"),
            index_dir: PathBuf::from("/index_dir"),
            cooldown_time: Duration::from_secs(60),
            notifications_addr: "0.0.0.0:8001".parse()?,
            websocket_cleanup_time: Duration::from_secs(10),
        };
        let loader = FsConfigLoader;

        // when
        loader.store(&cfg_path, &cfg)?;

        // then
        assert_eq!(
            read_to_string(&cfg_path)?,
            r#"watched_dir = "/watched_dir"
thumbnails_dir = "/thumbnails_dir"
index_dir = "/index_dir"
notifications_addr = "0.0.0.0:8001"

[cooldown_time]
secs = 60
nanos = 0

[websocket_cleanup_time]
secs = 10
nanos = 0
"#
        );

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Can't use '/' as a configuration path")]
    fn test_store_config_with_root_as_path() {
        // given
        let cfg_path = PathBuf::from("/");
        let cfg = Config::default();
        let loader = FsConfigLoader;

        // then
        loader.store(&cfg_path, &cfg).unwrap();
    }
}
