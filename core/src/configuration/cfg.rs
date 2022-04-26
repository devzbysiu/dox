use crate::result::{DoxErr, Result};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::create_dir_all;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::instrument;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Config {
    pub watched_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub index_dir: PathBuf,
    pub notifications_addr: SocketAddrV4,
    pub cooldown_time: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watched_dir: PathBuf::from(""),
            thumbnails_dir: thumbnails_dir_default(),
            index_dir: index_dir_default(),
            cooldown_time: Duration::from_secs(60),
            notifications_addr: "0.0.0.0:8001".parse().unwrap(),
        }
    }
}

#[instrument]
pub fn load<P: AsRef<Path> + Debug>(config_path: P) -> Result<Config> {
    Ok(toml::from_str(&read_to_string(config_path)?)?)
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

#[instrument]
pub fn store<P: AsRef<Path> + Debug>(path: P, cfg: &Config) -> Result<()> {
    let config_dir = path
        .as_ref()
        .parent()
        .ok_or_else(|| DoxErr::InvalidConfigPath("Can't use '/' as a configuration path".into()))?;
    create_dir_all(config_dir)?;
    let mut file = File::create(path)?;
    file.write_all(toml::to_string(cfg)?.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_default_config() -> Result<()> {
        // given
        let cfg = Config {
            watched_dir: PathBuf::from(""),
            thumbnails_dir: dirs::data_dir().unwrap().join("dox/thumbnails"),
            index_dir: dirs::data_dir().unwrap().join("dox/index"),
            cooldown_time: Duration::from_secs(60),
            notifications_addr: "0.0.0.0:8001".parse()?,
        };

        // when
        let default_cfg = Config::default();

        // then
        assert_eq!(cfg, default_cfg);

        Ok(())
    }

    #[test]
    fn test_read_config() -> Result<()> {
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
            "#,
        )?;
        let expected = Config {
            watched_dir: PathBuf::from("/home/zbyniu/Tests/notify"),
            thumbnails_dir: PathBuf::from("/home/zbyniu/.local/share/dox/thumbnails"),
            index_dir: PathBuf::from("/home/zbyniu/.local/share/dox/index"),
            cooldown_time: Duration::from_secs(1),
            notifications_addr: "0.0.0.0:8001".parse()?,
        };

        // when
        let read_cfg = load(cfg_path)?;

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
    fn test_read_config_when_missing_watched_dir() {
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
            "#,
        )
        .unwrap();

        // then
        load(cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `thumbnails_dir`")]
    fn test_read_config_when_missing_thumbnails_dir() {
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
            "#,
        )
        .unwrap();

        // then
        load(cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `index_dir`")]
    fn test_read_config_when_missing_index_dir() {
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
            "#,
        )
        .unwrap();

        // then
        load(cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `notifications_addr`")]
    fn test_read_config_when_missing_notifications_addr() {
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
            "#,
        )
        .unwrap();

        // then
        load(cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `cooldown_time`")]
    fn test_read_config_when_missing_cooldown_time() {
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
            "#,
        )
        .unwrap();

        // then
        load(cfg_path).unwrap(); // should panic
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
        };

        // when
        store(&cfg_path, &cfg)?;

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

        // then
        store(&cfg_path, &cfg).unwrap();
    }
}
