#![allow(clippy::missing_errors_doc)]

use anyhow::{bail, Result};
use log::debug;
use rocket::serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Deserialize, Default)]
pub struct SearchResults {
    pub entries: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
pub struct SearchEntry {
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct TestConfig {
    pub watched_dir: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub index_dir: PathBuf,
    pub cooldown_time: Duration,
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

pub fn create_cfg_file(cfg: &TestConfig) -> Result<TempDir> {
    let config_dir = tempfile::tempdir()?;
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
    let res = ureq::get(&url).call()?.into_json()?;
    debug!("search results: {:?}", res);
    Ok(res)
}

pub fn cp_docs<P: AsRef<Path>>(watched_dir: P) -> Result<()> {
    debug!("copying docs to watched dir...");
    let watched_dir = watched_dir.as_ref();
    let from = Path::new("./res/doc1.png");
    debug!("\tfrom {} to {}", from.display(), watched_dir.display());
    fs::copy(from, &watched_dir.join("doc1.png"))?; // TODO: it should be just one file
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
