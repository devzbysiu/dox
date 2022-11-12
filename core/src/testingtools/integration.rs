#![allow(unused)] // TODO: remove this

use super::{index_dir_path, thumbnails_dir_path, watched_dir_path};

use crate::configuration::factories::Context;
use crate::startup::rocket;
use crate::use_cases::config::Config;

use anyhow::Result;
use once_cell::sync::OnceCell;
use retry::delay::Fixed;
use retry::{retry, OperationResult};
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::local::blocking::LocalResponse;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashSet;
use std::fs::{self, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::TempDir;
use thiserror::Error;
use tracing::debug;

pub fn create_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let ctx = Context::new(&config)?;
    let client = Client::tracked(rocket(ctx))?;
    Ok(App {
        client,
        _config: config,
    })
}

fn config_dir_path(config_dir: &TempDir) -> PathBuf {
    config_dir.path().join("dox.toml")
}

#[derive(Debug)]
struct TestConfig {
    value: Config,
    watched_dir: TempDir,
    thumbnails_dir: TempDir,
    index_dir: TempDir,
}

impl TestConfig {
    fn new() -> Result<Self> {
        let watched_dir = watched_dir_path()?;
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
                thumbnails_dir: thumbnails_dir.path().to_path_buf(),
                index_dir: index_dir.path().to_path_buf(),
            },
            watched_dir,
            thumbnails_dir,
            index_dir,
        })
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

fn create_cfg_file(cfg: &TestConfig) -> Result<TempDir> {
    let mut dirs = tmp_dirs().lock().expect("poisoned mutex");
    let config_dir = tempfile::tempdir()?;
    let dir = config_dir.path().to_string_lossy().to_string();
    if dirs.contains(&dir) {
        panic!("Can't use the same directory");
    }
    dirs.insert(dir);
    drop(dirs);
    let config_path = config_path(&config_dir);
    let config = toml::to_string(&cfg)?;
    let mut file = fs::File::create(&config_path)?;
    debug!("writing {} to {}", config, config_path.display());
    file.write_all(config.as_bytes())?;
    Ok(config_dir)
}

fn tmp_dirs() -> &'static Mutex<HashSet<String>> {
    static INSTANCE: OnceCell<Mutex<HashSet<String>>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(HashSet::new()))
}

#[inline]
fn config_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join("dox.toml")
}

pub struct App {
    client: Client,
    _config: TestConfig,
}

impl App {
    pub fn search<S: Into<String>>(&self, q: S) -> Result<ApiResponse> {
        let mut resp = self
            .client
            .get(format!("/search?q={}", q.into()))
            .dispatch();
        let body = resp.read_body()?;
        Ok(ApiResponse {
            status: resp.status(),
            body,
        })
    }
}

pub struct ApiResponse {
    pub status: Status,
    pub body: String,
}

trait LocalResponseExt {
    fn read_body(&mut self) -> Result<String, HelperErr>;
}

impl LocalResponseExt for LocalResponse<'_> {
    fn read_body(&mut self) -> Result<String, HelperErr> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        let res = String::from_utf8(buffer)?;
        debug!("read the whole buffer: '{}'", res);
        Ok(res)
    }
}

trait ClientExt {
    fn read_entries(&self, endpoint: &str) -> Result<(String, Status), HelperErr>;
}

impl ClientExt for Client {
    fn read_entries(&self, endpoint: &str) -> Result<(String, Status), HelperErr> {
        Ok(retry(Fixed::from_millis(1000).take(60), || {
            let mut r = self.get(endpoint).dispatch();
            match r.read_body() {
                Ok(b) if b == r#"{"entries":[]}"# => OperationResult::Retry(("Empty", r.status())),
                Ok(b) if b.is_empty() => OperationResult::Retry(("Empty", r.status())),
                Ok(b) => OperationResult::Ok((b, r.status())),
                _ => OperationResult::Err(("Failed to fetch body", Status::InternalServerError)),
            }
        })
        .unwrap())
    }
}

// TODO: this probably shouldn't exist
#[derive(Debug, Error)]
pub enum HelperErr {
    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),

    #[error("Invalid utf characters.")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
