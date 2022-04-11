#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::cfg::Config;
use crate::extension::Ext;
use crate::extractor::ExtractorFactory;
use crate::helpers::PathRefExt;
use crate::indexer::{Repo, RepoTools};
use crate::preprocessor::PreprocessorFactory;
use crate::result::Result;
use crate::server::{all_thumbnails, receive_document, search};
use crate::user_input::handle_config;

use cooldown_buffer::cooldown_buffer;
use notifier::new_doc_notifier;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use rocket::fs::FileServer;
use rocket::{launch, routes, Build, Rocket};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing::{debug, error, instrument, warn};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::registry::Registry;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

mod cfg;
mod extension;
mod extractor;
mod helpers;
mod indexer;
mod notifier;
mod preprocessor;
mod prompt;
mod result;
mod server;
mod thumbnail;
mod user_input;

static TRACING: Lazy<()> = Lazy::new(|| setup_global_subscriber());

#[launch]
#[must_use]
pub fn launch() -> Rocket<Build> {
    let _guard = Lazy::force(&TRACING);

    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1));
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

fn setup_global_subscriber() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("dox".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[instrument]
fn setup(cfg: Config) -> Result<Repo> {
    debug!("setting up with config: {:?}", cfg);
    let doc_rx = spawn_watching_thread(&cfg);
    let repo_tools = indexer::mk_idx_and_schema(&cfg)?;
    spawn_indexing_thread(cfg, doc_rx, repo_tools.clone());
    Ok(Repo::new(repo_tools))
}

#[instrument]
fn spawn_watching_thread(cfg: &Config) -> Receiver<Vec<PathBuf>> {
    debug!("spawning watching thread");
    let (doc_tx, doc_rx) = cooldown_buffer(cfg.cooldown_time);
    let watched_dir = cfg.watched_dir.clone();
    thread::spawn(move || -> Result<()> {
        debug!("watching thread spawned");
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

#[instrument]
fn spawn_indexing_thread(cfg: Config, rx: Receiver<Vec<PathBuf>>, tools: RepoTools) {
    debug!("spawning indexing thread");
    thread::spawn(move || -> Result<()> {
        debug!("indexing thread spawned");
        let new_doc_notifier = new_doc_notifier(&cfg)?;
        loop {
            let paths = rx.recv()?;
            debug!("new docs: {:?}", paths);
            let extension = extension(&paths);
            PreprocessorFactory::from_ext(&extension, &cfg).preprocess(&paths)?;
            let tuples = ExtractorFactory::from_ext(&extension).extract_text(&paths);
            indexer::index_docs(&tuples, &tools)?;
            new_doc_notifier.notify()?;
        }
    });
}

fn extension(paths: &[PathBuf]) -> Ext {
    paths
        .first() // NOTE: I'm assuming the batched paths are all the same filetype
        .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
        .ext()
}
