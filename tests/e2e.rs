use anyhow::Result;
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
struct SearchResults {
    entries: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
struct SearchEntry {
    filename: String,
}

#[derive(Debug, Serialize)]
struct Config {
    watched_dir: PathBuf,
    index_dir: PathBuf,
    cooldown_time: Duration,
}

struct DoxProcess(Child);

impl Drop for DoxProcess {
    fn drop(&mut self) {
        self.0.kill().expect("failed to kill dox process");
    }
}

#[test]
fn it_works() -> Result<()> {
    pretty_env_logger::init();
    // given
    let index_dir = create_index_dir()?;
    let watched_dir = create_watched_dir()?;
    let config_dir = create_cfg_file(&Config {
        watched_dir: watched_dir.path().to_path_buf(),
        index_dir: index_dir.path().to_path_buf(),
        cooldown_time: Duration::from_secs(1),
    })?;

    let _dox_process = spawn_dox(config_path(&config_dir))?;

    let search = make_search("ale")?;

    assert!(search.entries.is_empty()); // initial search returns no results

    // when
    cp_docs(watched_dir.path())?; // then we copy documents and indexing starts

    // then
    let results = make_search("ale")?;

    let mut entries = results.entries;
    assert_eq!(entries.len(), 2); // then we have two results
    entries.sort_by(|a, b| a.filename.cmp(&b.filename));
    assert_eq!(
        entries,
        vec![
            SearchEntry {
                filename: "doc1.png".to_string()
            },
            SearchEntry {
                filename: "doc5.png".to_string()
            },
        ]
    );

    Ok(())
}

fn create_index_dir() -> Result<TempDir> {
    debug!("creating index directory");
    Ok(tempfile::tempdir()?)
}

fn create_watched_dir() -> Result<TempDir> {
    debug!("creating watched directory");
    Ok(tempfile::tempdir()?)
}

fn create_cfg_file(cfg: &Config) -> Result<TempDir> {
    let config_dir = tempfile::tempdir()?;
    let config_path = config_path(&config_dir);
    let config = toml::to_string(&cfg)?;
    let mut file = fs::File::create(&config_path)?;
    debug!("writing {} to {}", config, config_path.display());
    file.write_all(config.as_bytes())?;
    Ok(config_dir)
}

#[inline]
fn config_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join("dox.toml")
}

fn spawn_dox<P: AsRef<Path>>(config_path: P) -> Result<DoxProcess> {
    debug!("spawning 'dox {} &'", config_path.as_ref().display());
    let child = Command::new("./target/debug/dox")
        .arg(format!("{}", config_path.as_ref().display()))
        .arg("&")
        .spawn()?;
    thread::sleep(Duration::from_secs(2));
    Ok(DoxProcess(child))
}

fn make_search<S: Into<String>>(query: S) -> Result<SearchResults> {
    let url = format!("http://localhost:8000/search?q={}", query.into());
    let res = ureq::get(&url).call()?.into_json()?;
    debug!("search results: {:?}", res);
    Ok(res)
}

fn cp_docs<P: AsRef<Path>>(watched_dir: P) -> Result<()> {
    debug!("copying docs to watched dir...");
    let docs_dir = Path::new("./res");
    let watched_dir = watched_dir.as_ref();
    for file in fs::read_dir(docs_dir)? {
        let file = file?;
        let from = file.path();
        debug!("\tfrom {} to {}", from.display(), watched_dir.display());
        fs::copy(from, &watched_dir.join(file.file_name()))?;
    }
    debug!("done");
    thread::sleep(Duration::from_secs(10));
    Ok(())
}
