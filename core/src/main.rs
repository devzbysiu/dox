#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::cfg::Config;
use crate::extractor::{Ext, ExtractorFactory};
use crate::helpers::PathExt;
use crate::indexer::{Repo, RepoTools};
use crate::preprocessor::PreprocessorFactory;
use crate::result::Result;
use crate::server::{all_thumbnails, receive_document, search};
use crate::user_input::handle_config;

use cooldown_buffer::cooldown_buffer;
use log::{debug, error, warn};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rocket::fs::FileServer;
use rocket::{launch, routes, Build, Rocket};
use std::env;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tungstenite::{accept, WebSocket};

mod cfg;
mod extractor;
mod helpers;
mod indexer;
mod preprocessor;
mod prompt;
mod result;
mod server;
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
        .mount("/", routes![search, all_thumbnails, receive_document])
        .mount("/thumbnail", FileServer::from(&cfg.thumbnails_dir))
        .mount("/document", FileServer::from(&cfg.watched_dir))
        .manage(repo)
        .manage(cfg)
}

fn setup(cfg: Config) -> Result<Repo> {
    debug!("setting up with config: {:?}", cfg);
    let doc_rx = spawn_watching_thread(&cfg);
    let repo_tools = indexer::mk_idx_and_schema(&cfg)?;
    spawn_indexing_thread(cfg, doc_rx, repo_tools.clone());
    Ok(Repo::new(repo_tools))
}

fn spawn_watching_thread(cfg: &Config) -> Receiver<Vec<PathBuf>> {
    let (doc_tx, doc_rx) = cooldown_buffer(cfg.cooldown_time);
    let watched_dir = cfg.watched_dir.clone();
    thread::spawn(move || -> Result<()> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(100))?;
        watcher.watch(watched_dir, RecursiveMode::Recursive)?;
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Create(path)) => doc_tx.send(path)?,
                Ok(e) => warn!("this FS event is not supported: {:?}", e),
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });
    doc_rx
}

fn spawn_indexing_thread(cfg: Config, rx: Receiver<Vec<PathBuf>>, tools: RepoTools) {
    thread::spawn(move || -> Result<()> {
        loop {
            let paths = rx.recv()?;
            debug!("new docs: {:?}", paths);
            let extension = extension(&paths);
            PreprocessorFactory::from_ext(&extension, &cfg).preprocess(&paths)?;
            let tuples = ExtractorFactory::from_ext(&extension).extract_text(&paths);
            indexer::index_docs(&tuples, &tools.index, &tools.schema)?;
        }
    });
}

fn extension(paths: &[PathBuf]) -> Ext {
    paths
        .first() // NOTE: I'm assuming the batched paths are all the same filetype
        .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
        .ext()
}

fn notifications_channel() -> std::result::Result<Notifier, Box<dyn std::error::Error>> {
    let server = TcpListener::bind("0.0.0.0:8001")?;
    let stream = server.accept()?;
    let websocket = accept(stream.0)?;
    Ok(Notifier::new(websocket))
}

struct Notifier {
    websocket: WebSocket<TcpStream>,
}

impl Notifier {
    fn new(websocket: WebSocket<TcpStream>) -> Self {
        Self { websocket }
    }
}
