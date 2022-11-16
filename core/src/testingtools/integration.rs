#![allow(unused)] // TODO: remove this

use super::unit::Spy;
use super::{index_dir_path, thumbnails_dir_path, watched_dir_path};

use crate::configuration::factories::{repository, Context};
use crate::entities::document::DocDetails;
use crate::entities::location::SafePathBuf;
use crate::result::{FsErr, IndexerErr};
use crate::startup::rocket;
use crate::use_cases::config::Config;
use crate::use_cases::fs::{Filesystem, Fs};
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
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::{self, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use thiserror::Error;
use tracing::debug;
use urlencoding::encode;

pub fn start_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let ctx = Context::new(&config)?;
    let client = Client::tracked(rocket(ctx))?;
    Ok(App {
        client,
        config,
        tracked_repo_spy: None,
    })
}

pub fn test_app() -> Result<AppBuilder> {
    let config = TestConfig::new()?;
    let ctx = Context::new(&config)?;
    Ok(AppBuilder {
        config: Some(config),
        ctx: Some(ctx),
        tracked_repo_spy: None,
    })
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

pub struct App {
    client: Client,
    config: TestConfig,
    tracked_repo_spy: Option<Spy>,
}

impl App {
    pub fn wait_til_indexed(&mut self) {
        let spy = self.tracked_repo_spy();
        spy.method_called();
    }

    fn tracked_repo_spy(&self) -> &Spy {
        self.tracked_repo_spy
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized tracked repo spy"))
    }

    pub fn search<S: Into<String>>(&self, q: S) -> Result<ApiResponse> {
        let q = q.into();
        let urlencoded = encode(&q);
        self.client
            .get(format!("/search?q={}", urlencoded))
            .dispatch()
            .try_into()
    }

    pub fn upload_doc<P: AsRef<Path>>(&self, path: P) -> Result<ApiResponse> {
        let body = base64::encode(fs::read(&path)?);
        // TODO: cleanup getting the name
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        self.client
            .post("/document/upload")
            .body(
                json!({
                    "filename": filename,
                    "body": body
                })
                .to_string(),
            )
            .dispatch()
            .try_into()
    }

    pub fn get_doc<S: Into<String>>(&self, name: S) -> Result<ApiResponse> {
        let filename = name.into();
        self.client
            .get(format!("/document/{}", filename))
            .dispatch()
            .try_into()
    }

    pub fn get_thumbnail<S: Into<String>>(&self, name: S) -> Result<ApiResponse> {
        let filename = name.into();
        self.client
            .get(format!("/thumbnail/{}", filename))
            .dispatch()
            .try_into()
    }
}

pub struct ApiResponse {
    pub status: Status,
    pub body: String,
}

impl TryFrom<LocalResponse<'_>> for ApiResponse {
    type Error = anyhow::Error;

    fn try_from(mut res: LocalResponse<'_>) -> Result<Self, Self::Error> {
        Ok(ApiResponse {
            status: res.status(),
            body: res.read_body()?,
        })
    }
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

pub struct AppBuilder {
    config: Option<TestConfig>,
    ctx: Option<Context>,
    tracked_repo_spy: Option<Spy>,
}

impl AppBuilder {
    pub fn with_tracked_repo(mut self) -> Result<Self> {
        let cfg = self.config.as_ref().unwrap();
        let (spy, tracked_repo) = TrackedRepo::wrap(repository(cfg)?);
        let mut cfg = self.ctx.as_mut().unwrap();
        cfg.with_repo(tracked_repo);
        self.tracked_repo_spy = Some(spy);
        Ok(self)
    }

    pub fn with_failing_load_fs(mut self) -> Self {
        let cfg = self.config.as_ref().unwrap();
        let mut cfg = self.ctx.as_mut().unwrap();
        cfg.with_fs(FailingLoadFs::new());
        self
    }

    pub fn start(mut self) -> Result<App> {
        let client = Client::tracked(rocket(self.context()))?;
        let tracked_repo_spy = self.tracked_repo_spy();
        let config = self.config();
        Ok(App {
            client,
            config,
            tracked_repo_spy,
        })
    }

    fn context(&mut self) -> Context {
        self.ctx
            .take()
            .unwrap_or_else(|| panic!("uninitialized context"))
    }

    fn tracked_repo_spy(&mut self) -> Option<Spy> {
        self.tracked_repo_spy.take()
    }

    fn config(&mut self) -> TestConfig {
        self.config
            .take()
            .unwrap_or_else(|| panic!("uninitialized config"))
    }
}

pub fn doc<S: Into<String>>(name: S) -> PathBuf {
    let name = name.into();
    PathBuf::from(format!("res/{}", name))
}

impl Context {
    pub fn with_repo(&mut self, (repo_read, repo_write): (RepoRead, RepoWrite)) -> &Self {
        self.repo = (repo_read, repo_write);
        self
    }

    pub fn with_fs(&mut self, fs: Fs) -> &Self {
        self.fs = fs;
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

pub struct FailingLoadFs;

impl FailingLoadFs {
    fn new() -> Arc<Self> {
        Arc::new(FailingLoadFs)
    }
}

impl Filesystem for FailingLoadFs {
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        Ok(())
    }

    fn load(&self, uri: PathBuf) -> Result<Option<Vec<u8>>, FsErr> {
        Err(FsErr::TestError)
    }

    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        Ok(())
    }

    fn exists(&self, _path: &Path) -> bool {
        true
    }
}
