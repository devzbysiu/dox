use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::receiver::{DocsEvent, EventRecv};

use std::path::PathBuf;
use std::thread;
use tracing::{debug, error, warn};

/// Watches for the event comming from [`Watcher`] and publishes appropriate event on the event bus.
///
/// It then spawns new thread in which it receives events from [`Watcher`]. If the event is
/// [`WatcherEvent::Created`], then [`Event::NewDocs`] is created out of it and published on the
/// bus.
#[derive(Debug)]
pub struct DocsWatcher<'a> {
    bus: &'a dyn Bus,
}

impl<'a> DocsWatcher<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    pub fn run(&self, receiver: EventRecv) {
        debug!("spawning watching thread");
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            loop {
                debug!("waiting for event from watcher");
                match receiver.recv() {
                    Ok(DocsEvent::Created(path)) if path.is_file() => {
                        debug!("got create file event on path: '{}'", path.display());
                        publ.send(new_docs_event(path))?;
                    }
                    Ok(e) => warn!("this event is not supported: {:?}", e),
                    Err(e) => error!("watcher error: {:?}", e),
                }
            }
        });
    }
}

fn new_docs_event(path: PathBuf) -> BusEvent {
    debug!("new doc appeared, creating NewDocs event");
    BusEvent::NewDocs(Location::FileSystem(vec![path]))
}

// TODO: Fix this
// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::configuration::telemetry::init_tracing;
//     use crate::data_providers::bus::LocalBus;

//     use anyhow::Result;
//     use std::fs::File;
//     use std::io::Write;
//     use std::thread;
//     use tempfile::tempdir;

// #[test]
// fn test_fs_watcher_with_writing_to_file() -> Result<()> {
//     // given
//     init_tracing();
//     let tmp_dir = tempdir()?;
//     let bus = LocalBus::new()?;
//     let cfg = Config {
//         watched_dir: tmp_dir.path().into(),
//         ..Config::default()
//     };
//     let watcher = FsWatcher::new(&bus);
//     let sub = bus.subscriber();
//     let file_path = tmp_dir.path().join("test-file");

//     // when
//     watcher.run();
//     thread::sleep(Duration::from_secs(2));
//     let mut file = File::create(&file_path)?;
//     thread::sleep(Duration::from_secs(2)); // wait for Created event to be ignored
//     file.write_all(b"test")?;

//     let event = sub.recv()?;

//     // then
//     assert_eq!(event, Event::NewDocs(Location::FileSystem(vec![file_path])));

//     Ok(())
// }
// }
