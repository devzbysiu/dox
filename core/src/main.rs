#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::cfg::Config;
use crate::error::{DoxError, Result};
use crate::helpers::DirEntryExt;
use crate::index::{index_docs, mk_idx_and_schema, Repo, SearchResults};

use cooldown_buffer::cooldown_buffer;
use index::SearchEntry;
use log::{debug, error, warn};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, launch, post, routes, Build, Rocket, State};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

mod cfg;
mod error;
mod helpers;
mod index;
mod ocr;

#[launch]
fn launch() -> Rocket<Build> {
    pretty_env_logger::init();

    let args = env::args().collect::<Vec<String>>();
    let config_path = args
        .get(1)
        .expect("you need to specify the path to the configuration file");
    let cfg = cfg::read_config(config_path).expect("failed to read config");

    let repo = setup(&cfg).expect("failed to setup indexer");
    debug!("starting server...");
    rocket::build()
        .mount("/", routes![search, all_documents, receive_document])
        .mount("/document", FileServer::from(&cfg.watched_dir))
        .manage(repo)
        .manage(cfg)
}

fn setup(cfg: &Config) -> Result<Repo> {
    debug!("setting up with config: {:?}", cfg);
    let (doc_tx, doc_rx) = cooldown_buffer(cfg.cooldown_time);
    let watched_dir = cfg.watched_dir.clone();
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

    let (index, schema) = mk_idx_and_schema(&cfg.index_dir)?;

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
fn search(q: String, repo: &State<Repo>) -> Result<Json<SearchResults>> {
    Ok(Json(repo.search(q)?))
}

#[get("/documents/all")]
fn all_documents(cfg: &State<Config>) -> Result<Json<SearchResults>> {
    debug!("listing files from '{}':", cfg.watched_dir.display());
    let mut documents = Vec::new();
    for file in cfg.watched_dir.read_dir()? {
        let file = file?;
        let filename = file.filename();
        debug!("\t- {}", filename);
        documents.push(SearchEntry::new(filename));
    }
    Ok(Json(SearchResults::new(documents)))
}

#[derive(Deserialize)]
struct Document {
    filename: String,
    body: String,
}

#[post("/document/upload", data = "<doc>")]
async fn receive_document(doc: Json<Document>, cfg: &State<Config>) -> Result<Status> {
    debug!("receiving document: {}", doc.filename);
    let document = create_file(cfg.watched_dir.join(&doc.filename))?;
    write(document, &decode(&doc.body)?)?;
    Ok(Status::Created)
}

fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
    File::create(path).map_err(DoxError::Io)
}

fn decode<S: Into<String>>(body: S) -> Result<Vec<u8>> {
    base64::decode(body.into()).map_err(DoxError::Decode)
}

fn write(mut file: File, body: &[u8]) -> Result<()> {
    file.write_all(body).map_err(DoxError::Io)
}
