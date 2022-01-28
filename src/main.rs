#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::index::{index_docs, mk_idx_and_schema, Repo, SearchResults};

use anyhow::{Error, Result};
use cooldown_buffer::cooldown_buffer;
use log::{debug, error, warn};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rocket::response::Debug;
use rocket::serde::json::Json;
use rocket::{get, launch, routes, Build, Rocket, State};
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

mod index;
mod ocr;

#[derive(Debug, Deserialize)]
struct Config {
    watched_dir: PathBuf,
    index_dir: PathBuf,
    cooldown_time: Duration,
}

#[launch]
fn launch() -> Rocket<Build> {
    pretty_env_logger::init();

    let cfg = read_config().expect("failed to read config");

    let repo = setup(cfg).expect("failed to setup indexer");
    debug!("starting server...");
    rocket::build().mount("/", routes![search]).manage(repo)
}

fn read_config() -> Result<Config> {
    let config_path = dirs::config_dir().unwrap().join("dox");
    Ok(toml::from_str(&read_to_string(
        config_path.join("dox.toml"),
    )?)?)
}

fn setup(cfg: Config) -> Result<Repo> {
    let (doc_tx, doc_rx) = cooldown_buffer(cfg.cooldown_time);
    let watched_dir = cfg.watched_dir;
    thread::spawn(move || -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(100))?;
        watcher.watch(watched_dir, RecursiveMode::Recursive)?;
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Create(path)) => doc_tx.send(path)?,
                Ok(_) => warn!("this FS event is not supported"),
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });

    let (index, schema) = mk_idx_and_schema(cfg.index_dir)?;

    let (thread_idx, thread_schema) = (index.clone(), schema.clone());
    thread::spawn(move || -> Result<()> {
        loop {
            let paths = doc_rx.recv()?;
            debug!("new docs: {:?}", paths);
            let tuples = ocr::extract_text(&paths);
            index_docs(&tuples, &thread_idx, &thread_schema)?;
        }
    });
    Ok(Repo::new(index, schema))
}

#[get("/search?<q>")]
fn search(q: String, repo: &State<Repo>) -> Result<Json<SearchResults>, Debug<Error>> {
    Ok(Json(repo.search(q)?))
}
