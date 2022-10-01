#![allow(unused)] // TODO: remove this

use crate::entities::location::SafePathBuf;
use crate::startup::rocket;
use crate::use_cases::bus::{BusEvent, EventSubscriber};

use anyhow::{anyhow, Result};
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
use std::fs;
use std::fs::create_dir_all;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use thiserror::Error;
use tracing::debug;

pub fn create_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let config_dir = create_cfg_file(&config)?;
    let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
    Ok(App {
        client,
        _config_dir: config_dir,
    })
}

fn config_dir_string(config_dir: &TempDir) -> String {
    config_dir
        .path()
        .join("dox.toml")
        .to_string_lossy()
        .to_string()
}

#[derive(Debug)]
struct TestConfig {
    watched_dir: TempDir,
    thumbnails_dir: TempDir,
    index_dir: TempDir,
}

impl TestConfig {
    fn new() -> Result<Self> {
        Ok(Self {
            watched_dir: watched_dir_path()?,
            thumbnails_dir: thumbnails_dir_path()?,
            index_dir: index_dir_path()?,
        })
    }
}

pub fn index_dir_path() -> Result<TempDir> {
    debug!("creating index directory");
    Ok(tempfile::tempdir()?)
}

pub fn watched_dir_path() -> Result<TempDir> {
    debug!("creating watched directory");
    Ok(tempfile::tempdir()?)
}

pub fn thumbnails_dir_path() -> Result<TempDir> {
    debug!("creating thumbnails directory");
    Ok(tempfile::tempdir()?)
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
    _config_dir: TempDir,
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

pub trait SubscriberExt {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent>;
}

impl SubscriberExt for EventSubscriber {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent> {
        let (done_tx, done_rx) = channel();
        let handle = thread::spawn(move || -> Result<()> {
            let event = self.recv()?;
            done_tx.send(event)?;
            Ok(())
        });

        match done_rx.recv_timeout(timeout) {
            Ok(event) => {
                if let Err(e) = handle.join() {
                    panic!("failed to join thread: {:?}", e);
                }
                Ok(event)
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}

pub fn mk_file(user_dir_name: String, filename: String) -> Result<NewFile> {
    let tmp_dir = tempdir()?;
    let user_dir = tmp_dir.path().join(user_dir_name);
    create_dir_all(&user_dir)?;
    let path = user_dir.join(filename);
    fs::write(&path, "anything")?;
    let path = SafePathBuf::new(path);
    Ok(NewFile {
        _temp_dir: tmp_dir,
        path,
    })
}

pub struct NewFile {
    _temp_dir: TempDir,
    pub path: SafePathBuf,
}

pub struct Spy {
    rx: Receiver<()>,
}

impl Spy {
    pub fn new(rx: Receiver<()>) -> Self {
        Self { rx }
    }

    pub fn method_called(&self) -> bool {
        self.rx.recv_timeout(Duration::from_secs(2)).is_ok()
    }
}
