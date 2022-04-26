use crate::configuration::cfg::Config;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::pipe::{Event, Output};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct FsWatcher;

impl FsWatcher {
    pub fn run(cfg: &Config, output: Box<dyn Output>) {
        debug!("spawning watching thread");
        let watched_dir = cfg.watched_dir.clone();
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            let (watcher_tx, watcher_rx) = channel();
            let mut watcher = watcher(watcher_tx, Duration::from_millis(100))?;
            watcher.watch(watched_dir, RecursiveMode::Recursive)?;
            loop {
                match watcher_rx.recv() {
                    Ok(DebouncedEvent::Create(path)) => {
                        output.send(Event::NewDocs(Location::FileSystem(vec![path])))?
                    }
                    Ok(e) => warn!("this FS event is not supported: {:?}", e),
                    Err(e) => error!("watch error: {:?}", e),
                }
            }
        });
    }
}