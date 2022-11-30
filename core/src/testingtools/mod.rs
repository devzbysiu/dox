use crate::entities::location::SafePathBuf;
use crate::entities::user::{User, FAKE_USER_EMAIL};
use crate::result::FsErr;
use crate::use_cases::config::Config;
use crate::use_cases::fs::Filesystem;

use anyhow::Result;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{debug, instrument};

pub mod integration;
pub mod unit;

pub type WorkingFs = NoOpFs;

pub fn index_dir_path() -> Result<TempDir> {
    debug!("creating index directory");
    Ok(tempfile::tempdir()?)
}

pub fn watched_dir_path() -> Result<TempDir> {
    debug!("creating watched directory");
    Ok(tempfile::tempdir()?)
}

pub fn docs_dir_path() -> Result<TempDir> {
    debug!("creating docs directory");
    Ok(tempfile::tempdir()?)
}

pub fn thumbnails_dir_path() -> Result<TempDir> {
    debug!("creating thumbnails directory");
    Ok(tempfile::tempdir()?)
}

pub struct Spy {
    rx: Receiver<()>,
}

impl Spy {
    pub fn new(rx: Receiver<()>) -> Self {
        Self { rx }
    }

    pub fn method_called(&self) -> bool {
        self.rx.recv_timeout(Duration::from_secs(30)).is_ok()
    }
}

pub struct NoOpFs;

impl NoOpFs {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Filesystem for NoOpFs {
    #[instrument(skip(self))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        // nothing to do
        Ok(Vec::new())
    }

    #[instrument(skip(self))]
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }

    #[instrument(skip(self))]
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }
}

#[derive(Debug)]
pub struct TestConfig {
    value: Config,
    watched_dir: TempDir,
    #[allow(unused)]
    docs_dir: TempDir,
    thumbnails_dir: TempDir,
    index_dir: TempDir,
}

impl TestConfig {
    pub fn new() -> Result<Self> {
        let watched_dir = watched_dir_path()?;
        let docs_dir = docs_dir_path()?;
        let thumbnails_dir = thumbnails_dir_path()?;
        let index_dir = index_dir_path()?;
        Ok(Self {
            // NOTE: This weird 'config in config' is here because:
            // 1. I can't drop `TestConfig` - because it holds TempDir.
            // 2. TestConfig is useful on it's own (for example keeps in check the fields
            //    in `Config`).
            // 3. I need to use `Config` to build `Context`.
            // 4. I tried to introduce trait for configuration, but it requires implementing
            //    serde's `Serialize` and `Deserialize` traits which is not worth it.
            value: Config {
                watched_dir: watched_dir.path().to_path_buf(),
                docs_dir: docs_dir.path().to_path_buf(),
                thumbnails_dir: thumbnails_dir.path().to_path_buf(),
                index_dir: index_dir.path().to_path_buf(),
            },
            watched_dir,
            docs_dir,
            thumbnails_dir,
            index_dir,
        })
    }

    pub fn thumbnail_path<S: Into<String>>(&self, name: S) -> PathBuf {
        self.value.thumbnail_path(&User::new(FAKE_USER_EMAIL), name)
    }

    pub fn doc_path<S: Into<String>>(&self, name: S) -> PathBuf {
        self.value.document_path(&User::new(FAKE_USER_EMAIL), name)
    }
}

impl AsRef<Config> for TestConfig {
    fn as_ref(&self) -> &Config {
        &self.value
    }
}

impl Serialize for TestConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TestConfig", 5)?;
        state.serialize_field("watched_dir", self.watched_dir.path())?;
        state.serialize_field("thumbnails_dir", self.thumbnails_dir.path())?;
        state.serialize_field("index_dir", self.index_dir.path())?;
        state.end()
    }
}

impl From<TestConfig> for Config {
    fn from(cfg: TestConfig) -> Self {
        cfg.value
    }
}
