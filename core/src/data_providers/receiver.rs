use crate::result::DoxErr;
use crate::result::Result;
use crate::use_cases::receiver::{DocsEvent, EventReceiver};

use notify::RecommendedWatcher;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tracing::{error, warn};

pub struct FsEventReceiver {
    _watcher: RecommendedWatcher, // just keep watcher alive
    watcher_rx: Receiver<DebouncedEvent>,
}

impl FsEventReceiver {
    pub fn new<P: AsRef<Path>>(watched_dir: P) -> Result<Self> {
        let (watcher_tx, watcher_rx) = channel();
        let mut _watcher = watcher(watcher_tx, Duration::from_millis(100))?;
        _watcher.watch(watched_dir, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher,
            watcher_rx,
        })
    }
}

impl EventReceiver for FsEventReceiver {
    fn recv(&self) -> Result<DocsEvent> {
        match self.watcher_rx.recv() {
            Ok(DebouncedEvent::Create(path)) => Ok(DocsEvent::Created(path)),
            Ok(e) => {
                warn!("this FS event is not supported: {:?}", e);
                Ok(DocsEvent::Other)
            }
            Err(e) => {
                error!("watch error: {:?}", e);
                Err(DoxErr::Watcher(e))
            }
        }
    }
}
