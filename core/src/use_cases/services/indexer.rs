use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::repository::RepoWrite;

use std::thread;
use tracing::{debug, instrument, warn};

pub struct Indexer<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Indexer<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, repository))]
    pub fn run(&self, repository: RepoWrite) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::DataExtracted(doc_details) => {
                        repository.index(&doc_details)?;
                        publ.send(BusEvent::Indexed(doc_details))?;
                    }
                    e => debug!("event not supported in indexer: {}", e.to_string()),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::data_providers::bus::LocalBus;
    use crate::entities::document::DocDetails;
    use crate::use_cases::repository::RepositoryWrite;

    use anyhow::Result;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn repo_write_is_used_to_index_data() -> Result<()> {
        // given
        let (spy, repo_write) = RepoWriteSpy::new();
        let bus = LocalBus::new()?;

        // when
        let indexer = Indexer::new(&bus);
        indexer.run(repo_write);
        let mut publ = bus.publisher();
        publ.send(BusEvent::DataExtracted(Vec::new()))?;

        // then
        assert!(spy.index_called());

        Ok(())
    }

    struct RepoWriteSpy {
        tx: Mutex<Sender<()>>,
    }

    impl RepoWriteSpy {
        fn new() -> (Spy, Box<Self>) {
            let (tx, rx) = channel();
            (Spy::new(rx), Box::new(Self { tx: Mutex::new(tx) }))
        }
    }

    impl RepositoryWrite for RepoWriteSpy {
        fn index(&self, _docs_details: &[DocDetails]) -> crate::result::Result<()> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(())
        }
    }

    struct Spy {
        rx: Receiver<()>,
    }

    impl Spy {
        fn new(rx: Receiver<()>) -> Self {
            Self { rx }
        }

        fn index_called(&self) -> bool {
            self.rx.recv_timeout(Duration::from_secs(2)).is_ok()
        }
    }
}
