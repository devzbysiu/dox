use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::config::Config;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Watches for the changes on the File System and publishes correct event on the event bus.
#[derive(Debug)]
pub struct FsWatcher<'a> {
    cfg: &'a Config,
    bus: &'a dyn Bus,
}

impl<'a> FsWatcher<'a> {
    pub fn new(cfg: &'a Config, bus: &'a dyn Bus) -> Self {
        Self { cfg, bus }
    }

    pub fn run(&self) {
        debug!("spawning watching thread");
        let watched_dir = self.cfg.watched_dir.clone();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            let (watcher_tx, watcher_rx) = channel();
            let mut watcher = watcher(watcher_tx, Duration::from_millis(100))?;
            watcher.watch(watched_dir, RecursiveMode::Recursive)?;
            loop {
                debug!("waiting for event from watcher");
                match watcher_rx.recv() {
                    Ok(DebouncedEvent::Create(path)) => {
                        publ.send(Event::NewDocs(Location::FileSystem(vec![path])))?;
                    }
                    Ok(e) => warn!("this FS event is not supported: {:?}", e),
                    Err(e) => error!("watch error: {:?}", e),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{configuration::telemetry::init_tracing, data_providers::bus::LocalBus};

    use anyhow::Result;
    use std::fs::File;
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn test_fs_watcher_with_file_creation() -> Result<()> {
        // given
        init_tracing();
        let tmp_dir = tempdir()?;
        let bus = LocalBus::new()?;
        let cfg = Config {
            watched_dir: tmp_dir.path().into(),
            ..Default::default()
        };
        let watcher = FsWatcher::new(&cfg, &bus);
        let sub = bus.subscriber();
        let file_path = tmp_dir.path().join("test-file");

        // when
        watcher.run();
        thread::sleep(Duration::from_secs(2));
        File::create(&file_path)?;

        let event = sub.recv()?;

        // then
        assert_eq!(event, Event::NewDocs(Location::FileSystem(vec![file_path])));

        Ok(())
    }
}
