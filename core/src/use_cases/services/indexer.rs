use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::repository::RepoWrite;

use rayon::ThreadPoolBuilder;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub struct Indexer<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Indexer<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, repository))]
    pub fn run(&self, repository: RepoWrite) -> Result<()> {
        // TODO: add threadpool to other services
        // TODO: think about num_threads
        // TODO: should threadpool be shared between services?
        // TODO: should threadpool have it's own abstraction here?
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::DataExtracted(doc_details) => {
                        if let Err(e) = tp.install(|| -> Result<()> {
                            repository.index(&doc_details)?;
                            publ.send(BusEvent::Indexed(doc_details))?;
                            Ok(())
                        }) {
                            error!("indexing failed: '{}'", e);
                        }
                    }
                    e => debug!("event not supported in indexer: {}", e),
                }
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::data_providers::bus::LocalBus;
    use crate::entities::document::DocDetails;
    use crate::result::DoxErr;
    use crate::testutils::SubscriberExt;
    use crate::use_cases::repository::RepositoryWrite;
    use crate::use_cases::user::User;

    use anyhow::Result;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::Mutex;
    use std::time::Duration;
    use tantivy::TantivyError;

    #[test]
    fn repo_write_is_used_to_index_data() -> Result<()> {
        // given
        let (spy, working_repo_write) = RepoWriteSpy::working();
        let bus = LocalBus::new()?;

        // when
        let indexer = Indexer::new(&bus);
        indexer.run(working_repo_write)?;
        let mut publ = bus.publisher();
        publ.send(BusEvent::DataExtracted(Vec::new()))?;

        // then
        assert!(spy.index_called());

        Ok(())
    }

    #[test]
    fn indexed_event_is_send_on_success() -> Result<()> {
        // given
        let repo_write = Box::new(NoOpRepoWrite);
        let bus = LocalBus::new()?;

        // when
        let indexer = Indexer::new(&bus);
        indexer.run(repo_write)?;
        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::DataExtracted(Vec::new()))?;

        let _event = sub.recv()?; // ignore DataExtracted event

        // then
        assert_eq!(sub.recv()?, BusEvent::Indexed(Vec::new()));

        Ok(())
    }

    #[test]
    fn indexed_event_contains_docs_details_received_from_data_extracted_event() -> Result<()> {
        // given
        let repo_write = Box::new(NoOpRepoWrite);
        let bus = LocalBus::new()?;
        let docs_details = vec![DocDetails::new(
            User::new("some@email.com"),
            "path",
            "body",
            "thumbnail",
        )];

        // when
        let indexer = Indexer::new(&bus);
        indexer.run(repo_write)?;
        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::DataExtracted(docs_details.clone()))?;

        let _event = sub.recv()?; // ignore DataExtracted event

        // then
        assert_eq!(sub.recv()?, BusEvent::Indexed(docs_details));

        Ok(())
    }

    #[test]
    fn no_event_is_send_when_indexing_error_occurs() -> Result<()> {
        // given
        let repo_write = Box::new(ErroneousRepoWrite);
        let bus = LocalBus::new()?;

        // when
        let indexer = Indexer::new(&bus);
        indexer.run(repo_write)?;
        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::DataExtracted(Vec::new()))?;

        let _event = sub.recv()?; // ignore DataExtracted event

        // then
        assert!(sub.try_recv(Duration::from_secs(2)).is_err());

        Ok(())
    }

    #[test]
    fn failure_during_indexing_do_not_kill_service() -> Result<()> {
        // given
        let (spy, failing_repo_write) = RepoWriteSpy::failing();
        let bus = LocalBus::new()?;

        let indexer = Indexer::new(&bus);
        indexer.run(failing_repo_write)?;
        let mut publ = bus.publisher();
        publ.send(BusEvent::DataExtracted(Vec::new()))?;
        assert!(spy.index_called());

        // TODO: think about what should be in given and when
        // when
        publ.send(BusEvent::DataExtracted(Vec::new()))?;

        // then
        assert!(spy.index_called());

        Ok(())
    }

    struct RepoWriteSpy;

    impl RepoWriteSpy {
        fn working() -> (Spy, RepoWrite) {
            let (tx, rx) = channel();
            (Spy::new(rx), WorkingRepoWrite::make(tx))
        }

        fn failing() -> (Spy, RepoWrite) {
            let (tx, rx) = channel();
            (Spy::new(rx), FailingRepoWrite::make(tx))
        }
    }

    struct WorkingRepoWrite {
        tx: Mutex<Sender<()>>,
    }

    impl WorkingRepoWrite {
        fn make(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl RepositoryWrite for WorkingRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> crate::result::Result<()> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(())
        }
    }

    struct FailingRepoWrite {
        tx: Mutex<Sender<()>>,
    }

    impl FailingRepoWrite {
        fn make(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl RepositoryWrite for FailingRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> crate::result::Result<()> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Err(DoxErr::Indexing(TantivyError::Poisoned))
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

    struct NoOpRepoWrite;

    impl RepositoryWrite for NoOpRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> crate::result::Result<()> {
            // nothing to do here
            Ok(())
        }
    }

    struct ErroneousRepoWrite;

    impl RepositoryWrite for ErroneousRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> crate::result::Result<()> {
            Err(DoxErr::Indexing(TantivyError::Poisoned))
        }
    }
}
