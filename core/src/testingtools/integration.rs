#![allow(unused)] // TODO: remove this

use crate::configuration::factories::{repository, Context};
use crate::entities::document::DocDetails;
use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::{FsErr, IndexerErr, SearchErr};
use crate::startup::rocket;
use crate::testingtools::unit::Spy;
use crate::testingtools::{index_dir_path, thumbnails_dir_path, watched_dir_path};
use crate::use_cases::config::Config;
use crate::use_cases::fs::{Filesystem, Fs};
use crate::use_cases::repository::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite, SearchResult,
};

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
        repo_spies: None,
    })
}

pub fn test_app() -> Result<AppBuilder> {
    let config = TestConfig::new()?;
    let ctx = Context::new(&config)?;
    Ok(AppBuilder {
        config: Some(config),
        ctx: Some(ctx),
        repo_spies: None,
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
    repo_spies: Option<RepoSpies>,
}

impl App {
    pub fn wait_til_indexed(&mut self) {
        self.repo_spies().write.method_called();
    }

    fn repo_spies(&self) -> &RepoSpies {
        self.repo_spies
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized tracked repo spy"))
    }

    pub fn search<S: Into<String>>(&self, q: S) -> Result<ApiResponse> {
        let q = q.into();
        self.get(format!("/search?q={}", encode(&q)))
    }

    fn get<S: Into<String>>(&self, url: S) -> Result<ApiResponse> {
        self.client.get(url.into()).dispatch().try_into()
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
        self.get(format!("/document/{}", name.into()))
    }

    pub fn get_thumbnail<S: Into<String>>(&self, name: S) -> Result<ApiResponse> {
        self.get(format!("/thumbnail/{}", name.into()))
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
    repo_spies: Option<RepoSpies>,
}

impl AppBuilder {
    pub fn with_tracked_repo(mut self) -> Result<Self> {
        let cfg = self.config.as_ref().unwrap();
        let (repo_spies, tracked_repo) = TrackedRepo::wrap(&repository(cfg)?);
        let mut cfg = self.ctx.as_mut().unwrap();
        cfg.with_repo(tracked_repo);
        self.repo_spies = Some(repo_spies);
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
        let repo_spies = self.repo_spies();
        let config = self.config();
        Ok(App {
            client,
            config,
            repo_spies,
        })
    }

    fn context(&mut self) -> Context {
        self.ctx
            .take()
            .unwrap_or_else(|| panic!("uninitialized context"))
    }

    fn repo_spies(&mut self) -> Option<RepoSpies> {
        self.repo_spies.take()
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
    pub fn with_repo(&mut self, repo: Repo) -> &Self {
        self.repo = repo;
        self
    }

    pub fn with_fs(&mut self, fs: Fs) -> &Self {
        self.fs = fs;
        self
    }
}

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    pub fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
        let (read_tx, read_rx) = channel();
        let (write_tx, write_rx) = channel();
        let read_tx = Mutex::new(read_tx);
        let write_tx = Mutex::new(write_tx);
        let read = Spy::new(read_rx);
        let write = Spy::new(write_rx);
        (
            RepoSpies { read, write },
            Box::new(Self {
                write: TrackedWrite::create(repo.write(), write_tx),
                read: TrackedRead::create(repo.read(), read_tx),
            }),
        )
    }
}

impl Repository for TrackedRepo {
    fn read(&self) -> RepoRead {
        self.read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.write.clone()
    }
}

pub struct TrackedRead {
    read: RepoRead,
    tx: Mutex<Sender<()>>,
}

impl TrackedRead {
    fn create(read: RepoRead, tx: Mutex<Sender<()>>) -> RepoRead {
        Arc::new(Self { read, tx })
    }
}

impl RepositoryRead for TrackedRead {
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr> {
        self.read.search(user, q)
    }

    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.read.all_docs(user)
    }
}

pub struct TrackedWrite {
    write: RepoWrite,
    tx: Mutex<Sender<()>>,
}

impl TrackedWrite {
    fn create(write: RepoWrite, tx: Mutex<Sender<()>>) -> RepoWrite {
        Arc::new(Self { write, tx })
    }
}

impl RepositoryWrite for TrackedWrite {
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        debug!("before indexing");
        self.write.index(docs_details)?;
        let tx = self.tx.lock().expect("poisoned mutex");
        tx.send(());
        debug!("after indexing");
        Ok(())
    }
}

pub struct RepoSpies {
    read: Spy,
    write: Spy,
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

    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        Err(FsErr::TestError)
    }

    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        Ok(())
    }
}
