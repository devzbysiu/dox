#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::cfg::Config;
use crate::extractor::ExtractorFactory;
use crate::helpers::{DirEntryExt, PathExt};
use crate::index::{index_docs, mk_idx_and_schema, Repo, SearchResults};
use crate::preprocessor::PreprocessorFactory;
use crate::result::Result;
use crate::user_input::handle_config;

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
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

mod cfg;
mod extractor;
mod helpers;
mod index;
mod preprocessor;
mod prompt;
mod result;
mod thumbnail;
mod user_input;

#[launch]
fn launch() -> Rocket<Build> {
    pretty_env_logger::init();

    let path_override = env::args().nth(1);
    let cfg = handle_config(path_override).expect("failed to get config");

    let config = cfg.clone();
    let repo = setup(config).expect("failed to setup indexer");
    debug!("starting server...");
    rocket::build()
        .mount("/", routes![search, all_documents, receive_document])
        .mount("/document", FileServer::from(&cfg.thumbnails_dir))
        .manage(repo)
        .manage(cfg)
}

fn setup(cfg: Config) -> Result<Repo> {
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
            // NOTE: I'm assuming the batched paths are all the same filetype
            let extension = paths[0].ext();
            let preprocessor = PreprocessorFactory::from_ext(&extension, &cfg);
            preprocessor.preprocess(&paths)?;
            let extractor = ExtractorFactory::from_ext(&extension);
            let tuples = extractor.extract_text(&paths);
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
    for file in cfg.thumbnails_dir.read_dir()? {
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

#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
fn receive_document(doc: Json<Document>, cfg: &State<Config>) -> Result<Status> {
    debug!("receiving document: {}", doc.filename);
    let mut document = File::create(cfg.watched_dir.join(&doc.filename))?;
    document.write_all(&base64::decode(&doc.body)?)?;
    Ok(Status::Created)
}
