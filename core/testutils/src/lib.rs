#![allow(clippy::missing_errors_doc)]

use anyhow::{bail, Result};
use once_cell::sync::OnceCell;
use rand::Rng;
use rocket::serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashSet;
use std::env;
use std::fs::{self, create_dir_all, File};
use std::io::{Read, Write};
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use tracing::debug;

#[derive(Debug, Deserialize, Default)]
pub struct SearchResults {
    pub entries: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
pub struct SearchEntry {
    pub filename: String,
}

pub fn create_test_env() -> Result<(TestConfig, TempDir)> {
    let config = TestConfig::new()?;
    let config_dir = create_cfg_file(&config)?;
    override_config_path(&config_dir.path().join("dox.toml"));
    Ok((config, config_dir))
}

#[derive(Debug)]
pub struct TestConfig {
    watched_dir: TempDir,
    thumbnails_dir: TempDir,
    index_dir: TempDir,
    notifications_addr: SocketAddrV4,
    cooldown_time: Duration,
    websocket_cleanup_time: Duration,
}

impl TestConfig {
    pub fn new() -> Result<Self> {
        Ok(Self {
            watched_dir: watched_dir_path()?,
            thumbnails_dir: thumbnails_dir_path()?,
            index_dir: index_dir_path()?,
            notifications_addr: random_addr(),
            cooldown_time: Duration::from_secs(1),
            websocket_cleanup_time: Duration::from_secs(10),
        })
    }

    pub fn watched_dir_path(&self) -> PathBuf {
        self.watched_dir.path().to_path_buf()
    }

    pub fn thumbnails_dir_path(&self) -> PathBuf {
        self.thumbnails_dir.path().to_path_buf()
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
        state.serialize_field("notifications_addr", &self.notifications_addr)?;
        state.serialize_field("cooldown_time", &self.cooldown_time)?;
        state.serialize_field("websocket_cleanup_time", &self.websocket_cleanup_time)?;
        state.end()
    }
}

pub struct DoxProcess(Child);

impl Drop for DoxProcess {
    fn drop(&mut self) {
        self.0.kill().expect("failed to kill dox process");
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

fn tmp_dirs() -> &'static Mutex<HashSet<String>> {
    static INSTANCE: OnceCell<Mutex<HashSet<String>>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn create_cfg_file(cfg: &TestConfig) -> Result<TempDir> {
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

#[inline]
pub fn config_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join("dox.toml")
}

pub fn spawn_dox<P: AsRef<Path>>(config_path: P) -> Result<DoxProcess> {
    debug!("spawning 'dox {} &'", config_path.as_ref().display());
    let child = Command::new("./target/debug/dox")
        .arg(format!("{}", config_path.as_ref().display()))
        .arg("&")
        .spawn()?;
    thread::sleep(Duration::from_secs(2));
    Ok(DoxProcess(child))
}

pub fn make_search<S: Into<String>>(query: S) -> Result<SearchResults> {
    let url = format!("http://localhost:8000/search?q={}", query.into());
    let res = ureq::get(&url)
        .set("authorization", &id_token()?)
        .call()?
        .into_json()?;
    debug!("search results: {:?}", res);
    Ok(res)
}

fn id_token() -> Result<String> {
    let res: IdToken = ureq::post("https://www.googleapis.com/oauth2/v4/token")
        .send_json(ureq::json!({
                "grant_type": "refresh_token",
                "client_id": env!("DOX_CLIENT_ID"),
                "client_secret": env!("DOX_CLIENT_SECRET"),
                "refresh_token": env!("DOX_REFRESH_TOKEN"),

        }))?
        .into_json()?;
    Ok(res.id_token)
}

#[derive(Default, Deserialize)]
struct IdToken {
    id_token: String,
}

pub fn cp_docs<P: AsRef<Path>>(parent_dir: P) -> Result<()> {
    debug!("copying docs to watched dir...");
    let parent_dir = parent_dir.as_ref();
    let from = "./res/doc1.png";
    let to = parent_dir.join("doc1.png");
    create_dir_all(to.parent().expect("failed to get parent"))?;
    thread::sleep(Duration::from_secs(1)); // allow to start listening for events on this new dir
    debug!("\tfrom {} to {}", from, to.display());
    fs::copy(from, to)?; // TODO: it should be just one file
    debug!("done");
    thread::sleep(Duration::from_secs(15));
    Ok(())
}

pub fn ls<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
    let dir = dir.as_ref();
    if !dir.is_dir() {
        bail!("I can list only directories");
    }
    let mut result = Vec::new();
    for path in dir.read_dir()? {
        let path = path?;
        result.push(path.file_name().to_str().unwrap().to_string());
    }
    result.sort();
    Ok(result)
}

pub fn override_config_path<P: AsRef<Path>>(override_path: P) {
    let override_path = override_path.as_ref();
    env::set_var("DOX_CONFIG_PATH", override_path.display().to_string());
}

pub fn random_addr() -> SocketAddrV4 {
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(8000..9000);
    format!("0.0.0.0:{}", port).parse().unwrap()
}

pub fn to_base64<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buff = Vec::new();
    file.read_to_end(&mut buff)?;
    Ok(base64::encode(&buff))
}
