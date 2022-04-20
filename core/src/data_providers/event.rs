use crate::configuration::cfg::Config;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::event::Emitter;
use crate::use_cases::event::Event;
use crate::use_cases::event::Sink;

use cooldown_buffer::cooldown_buffer;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct FsSink {
    doc_rx: Receiver<Vec<PathBuf>>,
}

impl FsSink {
    pub fn new(cfg: &Config) -> Self {
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
        Self { doc_rx }
    }
}

impl Sink for FsSink {
    fn recv(&self) -> Result<Event> {
        let paths = self.doc_rx.recv()?;
        Ok(Event::NewDocs(Location::FileSystem(paths)))
    }
}

#[derive(Debug, Clone)]
pub struct DefaultEmitter;

impl DefaultEmitter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Emitter for DefaultEmitter {
    fn send(&self, location: Location) -> Result<()> {
        unimplemented!()
    }
}

impl Sink for DefaultEmitter {
    fn recv(&self) -> Result<Event> {
        unimplemented!()
    }
}
