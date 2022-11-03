#![allow(unused)] // TODO: remove this

use crate::configuration::factories::event_bus;
use crate::entities::document::DocDetails;
use crate::entities::location::{Location, SafePathBuf};
use crate::entities::user::FAKE_USER_EMAIL;
use crate::startup::rocket;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};
use crate::use_cases::receiver::DocsEvent;

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
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use thiserror::Error;
use tracing::debug;

pub fn create_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let config_dir = create_cfg_file(&config)?;
    let client = Client::tracked(rocket(Some(config_dir_path(&config_dir))))?;
    Ok(App {
        client,
        _config_dir: config_dir,
    })
}

fn config_dir_path(config_dir: &TempDir) -> PathBuf {
    config_dir.path().join("dox.toml")
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

pub fn create_test_shim() -> Result<TestShim> {
    let test_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
    let bus = event_bus()?;
    let publ = bus.publisher();
    let sub = bus.subscriber();
    let (tx, rx) = channel();
    let rx = Some(rx);
    Ok(TestShim {
        rx,
        tx,
        test_file,
        bus,
        publ,
        sub,
    })
}

pub struct TestShim {
    rx: Option<Receiver<DocsEvent>>,
    tx: Sender<DocsEvent>,
    test_file: TestFile,
    bus: EventBus,
    publ: EventPublisher,
    sub: EventSubscriber,
}

impl TestShim {
    pub fn trigger_encrypter(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::EncryptionRequest(test_location))?;
        Ok(())
    }

    pub fn bus(&self) -> EventBus {
        self.bus.clone()
    }

    pub fn ignore_event(&self) -> Result<()> {
        let _event = self.sub.recv()?; // ignore message sent earliner
        Ok(())
    }

    pub fn pipeline_finished(&self) -> Result<bool> {
        let event = self.sub.recv()?;
        Ok(event == BusEvent::PipelineFinished)
    }

    pub fn no_events_on_bus(self) -> bool {
        self.sub.try_recv(Duration::from_secs(2)).is_err()
    }

    pub fn send_events(&mut self, events: &[BusEvent]) -> Result<()> {
        for event in events {
            self.publ.send(event.clone())?;
        }
        Ok(())
    }

    pub fn no_such_events(&self, ignored: &[BusEvent], max_events: usize) -> Result<bool> {
        for i in 0..max_events {
            let received = self.sub.recv()?;
            for event in ignored {
                if *event == received {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    pub fn trigger_extractor(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::NewDocs(test_location))?;
        Ok(())
    }

    pub fn event_on_bus(&self, event: &BusEvent) -> Result<bool> {
        Ok(*event == self.sub.recv()?)
    }

    pub fn test_location(&self) -> Location {
        self.test_file.location.clone()
    }

    // TODO: this should take data the indexer should be triggered with - do that also for other
    // trigger_* methods
    pub fn trigger_indexer(&mut self, details: Vec<DocDetails>) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::DataExtracted(details))?;
        Ok(())
    }

    pub fn trigger_preprocessor(&mut self) -> Result<()> {
        self.publ.send(BusEvent::NewDocs(self.test_location()))?;
        Ok(())
    }

    pub fn rx(&mut self) -> Receiver<DocsEvent> {
        self.rx.take().unwrap()
    }

    // TODO: I think it would be better to explicitly pass the event being sent - it's cleaner in
    // the test. This should be changed for all trigger_* methods
    pub fn trigger_watcher(&self) -> Result<()> {
        let file_path = self.test_file.path.clone();
        self.tx.send(DocsEvent::Created(file_path))?;
        Ok(())
    }

    pub fn mk_docs_event(&self, event: DocsEvent) -> Result<()> {
        self.tx.send(event)?;
        Ok(())
    }
}

fn mk_file(user_dir_name: String, filename: String) -> Result<TestFile> {
    let tmp_dir = tempdir()?;
    let user_dir = tmp_dir.path().join(user_dir_name);
    create_dir_all(&user_dir)?;
    let path = user_dir.join(filename);
    fs::write(&path, "anything")?;
    let path = SafePathBuf::new(path);
    Ok(TestFile {
        _temp_dir: tmp_dir,
        location: Location::FS(vec![path.clone()]),
        path,
    })
}

pub struct TestFile {
    _temp_dir: TempDir,
    pub path: SafePathBuf,
    pub location: Location,
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
