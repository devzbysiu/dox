use crate::entities::file::{Filename, Thumbnailname};
use crate::entities::user::{User, FAKE_USER_EMAIL};
use crate::use_cases::config::Config;

use anyhow::Result;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{debug, error};

pub mod api;
pub mod app;
pub mod services;
pub mod unit;

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

#[derive(Debug)]
pub struct TestConfig {
    value: Config,
    watched_dir: TempDir,
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
        let name = name.into();
        let thumbnailname = Thumbnailname::new(name).expect("Failed to create thumbnail name");
        self.value
            .thumbnail_path(&User::new(FAKE_USER_EMAIL), &thumbnailname)
    }

    pub fn doc_path<S: Into<String>>(&self, name: S) -> PathBuf {
        let name = name.into();
        let filename = Filename::new(name).expect("Failed to create filename");
        self.value
            .document_path(&User::new(FAKE_USER_EMAIL), &filename)
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

impl From<&TestConfig> for Config {
    fn from(cfg: &TestConfig) -> Self {
        cfg.value.clone()
    }
}

pub type Tx = Mutex<Sender<()>>;

pub fn pipe() -> (Tx, Spy) {
    let (tx, rx) = channel();
    (Mutex::new(tx), Spy::new(rx))
}

pub trait MutexExt {
    fn signal(&self);
}

impl MutexExt for Tx {
    fn signal(&self) {
        let tx = self.lock().expect("poisoned mutex");
        // NOTE: We can't `unwrap` or `expect` (etc.) here because during testing, the other end of
        // the channel gets dropped while this end is still used in thread. The result is that
        // `send` returns error and `unwrap` or `expect` panics which triigers abort and stop the
        // test binary.
        // This `signal` fn is used only for testing and it's acceptable to ignore this error.
        // Ultimately, if the other end is dropped it means that the test finished and all its
        // requirements are fullfilled.
        if let Err(e) = tx.send(()) {
            error!("failed to send signal: {:?}", e);
        }
    }
}
