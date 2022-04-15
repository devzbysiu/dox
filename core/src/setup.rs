use crate::cfg::Config;
use crate::entities::extension::Ext;
use crate::entities::extractor::ExtractorFactory;
use crate::entities::preprocessor::PreprocessorFactory;
use crate::helpers::PathRefExt;
use crate::indexer::{self, Repo, RepoTools};
use crate::notifier::new_doc_notifier;
use crate::result::Result;
use crate::use_cases::extractor::ExtractorFactoryImpl;
use crate::use_cases::preprocessor::PreprocessorFactoryImpl;

use cooldown_buffer::cooldown_buffer;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, instrument, warn};

#[instrument]
pub fn setup(cfg: Config) -> Result<Repo> {
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

#[instrument(skip(tools))]
fn spawn_indexing_thread(cfg: Config, rx: Receiver<Vec<PathBuf>>, tools: RepoTools) {
    debug!("spawning indexing thread");
    thread::spawn(move || -> Result<()> {
        debug!("indexing thread spawned");
        let new_doc_notifier = new_doc_notifier(&cfg)?;
        loop {
            let paths = rx.recv()?;
            debug!("new docs: {:?}", paths);
            let extension = extension(&paths);
            // TODO: do I need to pass Config here? Maybe I should introduce some kind of
            // 'arguments' (like map or something) in the preprocessor interface?
            PreprocessorFactoryImpl::from_ext(&extension, &cfg).preprocess(&paths)?;
            let tuples = ExtractorFactoryImpl::from_ext(&extension).extract_text(&paths);
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
