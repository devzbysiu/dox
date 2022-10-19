//! This is a concrete implementation of [`crate::use_cases::config`] mod.
use crate::data_providers::prompt;
use crate::helpers::PathRefExt;
use crate::result::{ConfigurationErr, PromptErr};
use crate::use_cases::config::{CfgLoader, Config, ConfigLoader, ConfigResolver};

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
    fn load(&self, path: &Path) -> Result<Config, ConfigurationErr> {
        Ok(toml::from_str(&read_to_string(path)?)?)
    }

    #[instrument(skip(self))]
    fn store(&self, path: &Path, cfg: &Config) -> Result<(), ConfigurationErr> {
        let config_dir = path.parent().ok_or_else(|| {
            ConfigurationErr::InvalidConfigPath("Can't use '/' as a configuration path".into())
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
    config_loader: CfgLoader,
}

impl FsConfigResolver {
    pub fn new(config_loader: CfgLoader) -> Self {
        Self { config_loader }
    }
}

impl ConfigResolver for FsConfigResolver {
    // TODO: path_override should be a PathBuf
    #[instrument(skip(self))]
    fn handle_config(&self, path_override: Option<String>) -> Result<Config, ConfigurationErr> {
        let config_path = path_override.map_or(config_path(), PathBuf::from);
        let cfg = if config_path.exists() {
            debug!("loading config from '{}'", config_path.str());
            let cfg = self.config_loader.load(&config_path)?;
            debug!("loaded config: {:?}", cfg);
            cfg
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

fn config_from_user() -> Result<Config, ConfigurationErr> {
    match prompt::show() {
        Ok(cfg) => Ok(cfg),
        Err(PromptErr::Prompt(InquireError::OperationCanceled)) => exit_process(),
        Err(e) => panic!("failed while showing prompt: {}", e),
    }
}

fn exit_process() -> ! {
    debug!("prompt cancelled, exiting process");
    std::process::exit(0);
}

fn prepare_directories(config: &Config) -> Result<(), ConfigurationErr> {
    debug!("preparing directories");
    check_watched_dir(config)?;
    check_thumnbails_dir(config)?;
    check_index_dir(config)?;
    Ok(())
}

fn check_watched_dir(config: &Config) -> Result<(), ConfigurationErr> {
    debug!("checking watched dir");
    if config.watched_dir.exists() && !config.watched_dir.is_dir() {
        return Err(ConfigurationErr::InvalidWatchedDirPath(format!(
            "It needs to be a directory: '{}'",
            config.watched_dir.str()
        )));
    }
    create_dir_all(&config.watched_dir)?;
    Ok(())
}

fn check_thumnbails_dir(config: &Config) -> Result<(), ConfigurationErr> {
    debug!("checking thumbnails dir");
    if config.thumbnails_dir.exists() && !config.thumbnails_dir.is_dir() {
        return Err(ConfigurationErr::InvalidIndexPath(format!(
            "It needs to be a directory: '{}'",
            config.thumbnails_dir.str()
        )));
    }
    create_dir_all(&config.thumbnails_dir)?;
    if config.thumbnails_dir.read_dir()?.next().is_some() {
        return Err(ConfigurationErr::InvalidThumbnailPath(format!(
            "Directory needs to be empty: '{}'",
            config.thumbnails_dir.str()
        )));
    }
    Ok(())
}

fn check_index_dir(config: &Config) -> Result<(), ConfigurationErr> {
    debug!("checking index dir");
    if config.index_dir.exists() && !config.index_dir.is_dir() {
        return Err(ConfigurationErr::InvalidIndexPath(format!(
            "It needs to be a directory: '{}'",
            config.index_dir.str()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::data_providers::config::config_path;
    use crate::testingtools::Spy;

    use anyhow::Result;
    use claim::assert_matches;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use std::fs::{self, read_to_string};
    use std::path::Path;
    use std::sync::mpsc::{channel, Sender};
    use tempfile::tempdir;

    #[test]
    fn config_is_loaded_properly_from_a_file() -> Result<()> {
        // given
        let tmp_cfg = tempdir()?;
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
            watched_dir = "/home/zbyniu/Tests/notify"
            thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
            index_dir = "/home/zbyniu/.local/share/dox/index"
            "#,
        )?;
        let expected = Config {
            watched_dir: PathBuf::from("/home/zbyniu/Tests/notify"),
            thumbnails_dir: PathBuf::from("/home/zbyniu/.local/share/dox/thumbnails"),
            index_dir: PathBuf::from("/home/zbyniu/.local/share/dox/index"),
        };
        let loader = FsConfigLoader;

        // when
        let read_cfg = loader.load(&cfg_path)?;

        // then
        assert_eq!(expected, read_cfg);

        Ok(())
    }

    fn create_config<A: AsRef<Path>, S: Into<String>>(path: A, content: S) -> Result<()> {
        let path = path.as_ref();
        let mut cfg_file = File::create(&path)?;
        cfg_file.write_all(content.into().as_bytes())?;
        Ok(())
    }

    #[test]
    #[should_panic(expected = "missing field `watched_dir`")]
    fn missing_watched_dir_in_config_causes_panic_in_config_loader() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `thumbnails_dir`")]
    fn missing_thumbnails_dir_causes_panic_in_config_loader() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                index_dir = "/home/zbyniu/.local/share/dox/index"
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "missing field `index_dir`")]
    fn missing_index_dir_causes_panic_in_config_loader() {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        create_config(
            &cfg_path,
            r#"
                watched_dir = "/home/zbyniu/Tests/notify"
                thumbnails_dir = "/home/zbyniu/.local/share/dox/thumbnails"
                "#,
        )
        .unwrap();
        let loader = FsConfigLoader;

        // then
        loader.load(&cfg_path).unwrap(); // should panic
    }

    #[test]
    fn config_path_returns_correct_path() {
        // given
        let path = dirs::config_dir().unwrap().join("dox/dox.toml");

        // when
        let cfg_path = config_path();

        // then
        assert_eq!(cfg_path, path);
    }

    #[test]
    fn config_is_saved_correctly_in_file() -> Result<()> {
        // given
        let tmp_cfg = tempdir().unwrap();
        let cfg_path = tmp_cfg.path().join("dox.toml");
        let cfg = Config {
            watched_dir: PathBuf::from("/watched_dir"),
            thumbnails_dir: PathBuf::from("/thumbnails_dir"),
            index_dir: PathBuf::from("/index_dir"),
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
"#
        );

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Can't use '/' as a configuration path")]
    fn config_can_not_be_stored_directly_under_root() {
        // given
        let cfg_path = PathBuf::from("/");
        let cfg = Config::default();
        let loader = FsConfigLoader;

        // then
        loader.store(&cfg_path, &cfg).unwrap();
    }

    #[test]
    fn config_resolver_loads_config_from_path_override() -> Result<()> {
        // given
        init_tracing();
        let tmp_cfg = tempdir()?;
        let cfg_path = tmp_cfg.path().join("dox.toml");
        let config = Config {
            watched_dir: tmp_cfg.path().join("watched_dir"),
            thumbnails_dir: tmp_cfg.path().join("thumbnails_dir"),
            index_dir: tmp_cfg.path().join("index_dir"),
        };
        let config_content = toml::to_string(&config)?;
        create_config(&cfg_path, &config_content)?;
        let (spy, loader) = ConfigLoaderSpy::create(config);
        let config_resolver = FsConfigResolver::new(loader);
        let path_override = Some(cfg_path.string());

        // when
        let _cfg = config_resolver.handle_config(path_override)?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn config_resolver_returns_invalid_watched_dir_err_path_when_it_is_not_a_dir() -> Result<()> {
        // given
        init_tracing();
        let tmp_cfg = tempdir()?;
        let cfg_path = tmp_cfg.path().join("dox.toml");
        let watched_dir = tmp_cfg.path().join("watched_dir");
        let dummy_file_content: String = Paragraph(1..2).fake();
        fs::write(&watched_dir, &dummy_file_content)?; // create file instead of directory
        let config = Config {
            watched_dir: watched_dir.clone(),
            thumbnails_dir: tmp_cfg.path().join("thumbnails_dir"),
            index_dir: tmp_cfg.path().join("index_dir"),
        };
        let config_content = toml::to_string(&config)?;
        create_config(&cfg_path, &config_content)?;
        let (spy, loader) = ConfigLoaderSpy::create(config);
        let config_resolver = FsConfigResolver::new(loader);
        let path_override = Some(cfg_path.string());
        let err_msg = format!("It needs to be a directory: '{}'", watched_dir.string());

        // when
        let res = config_resolver.handle_config(path_override);

        // then
        assert!(spy.method_called());
        assert_matches!(res, Err(ConfigurationErr::InvalidWatchedDirPath(msg)) if msg == err_msg);

        Ok(())
    }

    struct ConfigLoaderSpy;

    impl ConfigLoaderSpy {
        fn create(config: Config) -> (Spy, CfgLoader) {
            let (tx, rx) = channel();
            (Spy::new(rx), Loader::make(tx, config))
        }
    }

    struct Loader {
        tx: Sender<()>,
        config: Config,
    }

    impl Loader {
        fn make(tx: Sender<()>, config: Config) -> Box<Self> {
            Box::new(Self { tx, config })
        }
    }

    impl ConfigLoader for Loader {
        fn load(&self, _path: &Path) -> Result<Config, ConfigurationErr> {
            self.tx.send(()).expect("failed to send message");
            Ok(self.config.clone())
        }

        fn store(&self, _path: &Path, _cfg: &Config) -> Result<(), ConfigurationErr> {
            // nothing to do
            Ok(())
        }
    }
}
