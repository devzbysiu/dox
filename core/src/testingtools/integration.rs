#![allow(unused)] // TODO: remove this

use super::unit::Spy;
use super::{index_dir_path, thumbnails_dir_path, watched_dir_path};

use crate::configuration::factories::Context;
use crate::entities::document::DocDetails;
use crate::result::IndexerErr;
use crate::startup::rocket;
use crate::use_cases::config::Config;
use crate::use_cases::repository::{RepoRead, RepoWrite, RepositoryWrite};

use anyhow::Result;
use once_cell::sync::OnceCell;
use retry::delay::Fixed;
use retry::{retry, OperationResult};
use rocket::http::Status;
use rocket::local::blocking::Client;
use rocket::local::blocking::LocalResponse;
use rocket::serde::json::json;
use rocket::serde::Serialize;
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashSet;
use std::fs::{self, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use thiserror::Error;
use tracing::debug;
use urlencoding::encode;

pub fn test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let ctx = Context::new(&config)?;
    let client = Client::tracked(rocket(ctx))?;
    Ok(App {
        client,
        _config: config,
    })
}

pub fn test_app_with(ctx: Context, config: TestConfig) -> Result<App> {
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
pub struct TestConfig {
    value: Config,
    watched_dir: TempDir,
    thumbnails_dir: TempDir,
    index_dir: TempDir,
}

impl TestConfig {
    pub fn new() -> Result<Self> {
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
        let q = q.into();
        let urlencoded = encode(&q);
        let mut resp = self
            .client
            .get(format!("/search?q={}", urlencoded))
            .dispatch();
        let body = resp.read_body()?;
        Ok(ApiResponse {
            status: resp.status(),
            body,
        })
    }

    pub fn upload_doc<P: AsRef<Path>>(&self, path: P) -> Result<ApiResponse> {
        let body = base64::encode(fs::read(&path)?);
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let mut resp = self
            .client
            .post("/document/upload")
            .body(
                json!({
                    "filename": filename,
                    "body": body
                })
                .to_string(),
            )
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

pub fn doc<S: Into<String>>(name: S) -> PathBuf {
    let name = name.into();
    PathBuf::from(format!("res/{}", name))
}

impl Context {
    pub fn with_repo(mut self, (repo_read, repo_write): (RepoRead, RepoWrite)) -> Self {
        self.repo = (repo_read, repo_write);
        self
    }
}

pub struct TrackedRepo {
    repo_write: RepoWrite,
    tx: Mutex<Sender<()>>,
}

impl TrackedRepo {
    pub fn wrap((repo_read, repo_write): (RepoRead, RepoWrite)) -> (Spy, (RepoRead, RepoWrite)) {
        let (tx, rx) = channel();
        let tx = Mutex::new(tx);
        (Spy::new(rx), (repo_read, Arc::new(Self { repo_write, tx })))
    }
}

impl RepositoryWrite for TrackedRepo {
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        debug!("before indexing");
        self.repo_write.index(docs_details)?;
        let tx = self.tx.lock().expect("poisoned mutex");
        tx.send(());
        debug!("after indexing");
        Ok(())
    }
}
