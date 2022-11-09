use crate::result::IndexerErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::repository::RepoWrite;

use rayon::ThreadPoolBuilder;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub struct Indexer {
    bus: EventBus,
}

impl Indexer {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, repo))]
    pub fn run(&self, repo: RepoWrite) -> Result<(), IndexerErr> {
        // TODO: add threadpool to other services
        // TODO: think about num_threads
        // TODO: should threadpool be shared between services?
        // TODO: should threadpool have it's own abstraction here?
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<(), IndexerErr> {
            loop {
                match sub.recv()? {
                    BusEvent::DataExtracted(doc_details) => {
                        if let Err(e) = tp.install(|| -> Result<(), IndexerErr> {
                            repo.index(&doc_details)?;
                            publ.send(BusEvent::Indexed(doc_details))?;
                            Ok(())
                        }) {
                            error!("indexing failed: '{}'", e);
                        }
                    }
                    e => debug!("event not supported in indexer: '{}'", e),
                }
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::entities::document::DocDetails;
    use crate::result::{BusErr, IndexerErr};
    use crate::testingtools::{create_test_shim, Spy};
    use crate::use_cases::repository::RepositoryWrite;

    use anyhow::{anyhow, Result};
    use fake::{Fake, Faker};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::Mutex;

    #[test]
    fn repo_write_is_used_to_index_data() -> Result<()> {
        // given
        init_tracing();
        let (spy, working_repo_write) = RepoWriteSpy::working();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus()).run(working_repo_write)?;

        // when
        shim.trigger_indexer(Faker.fake())?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn indexed_event_is_send_on_success() -> Result<()> {
        // given
        init_tracing();
        let noop_repo_write = NoOpRepoWrite::new();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus()).run(noop_repo_write)?;
        let doc_details: Vec<DocDetails> = Faker.fake();

        // when
        shim.trigger_indexer(doc_details.clone())?;

        shim.ignore_event()?; // ignore DataExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::Indexed(doc_details))?);

        Ok(())
    }

    #[test]
    fn indexed_event_contains_docs_details_received_from_data_extracted_event() -> Result<()> {
        // given
        init_tracing();
        let repo_write = NoOpRepoWrite::new();
        let mut shim = create_test_shim()?;
        let docs_details: Vec<DocDetails> = Faker.fake();
        Indexer::new(shim.bus()).run(repo_write)?;

        // when
        shim.trigger_indexer(docs_details.clone())?;

        shim.ignore_event()?; // ignore DataExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::Indexed(docs_details))?);

        Ok(())
    }

    #[test]
    fn no_event_is_send_when_indexing_error_occurs() -> Result<()> {
        // given
        init_tracing();
        let repo_write = ErroneousRepoWrite::new();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus()).run(repo_write)?;

        // when
        shim.trigger_indexer(Faker.fake())?;

        shim.ignore_event()?; // ignore DataExtracted event

        // then
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn indexer_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let noop_repo_write = NoOpRepoWrite::new();
        let mut shim = create_test_shim()?;
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::EncryptDocument(Faker.fake()),
            BusEvent::EncryptThumbnail(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::PipelineFinished,
        ];
        Indexer::new(shim.bus()).run(noop_repo_write).unwrap();

        // when
        shim.send_events(&ignored_events)?;

        // then
        // all events are still on the bus, no Indexed emitted
        assert!(shim.no_such_events(&[BusEvent::Indexed(Vec::new())], ignored_events.len())?);
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_indexing_do_not_kill_service() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_repo_write) = RepoWriteSpy::failing();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus()).run(failing_repo_write)?;
        shim.trigger_indexer(Faker.fake())?;
        assert!(spy.method_called());

        // when
        shim.trigger_indexer(Faker.fake())?;

        // then
        assert!(spy.method_called());

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
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
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
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Err(IndexerErr::BusError(BusErr::GenericError(anyhow!("error"))))
        }
    }

    struct NoOpRepoWrite;

    impl NoOpRepoWrite {
        fn new() -> Box<Self> {
            Box::new(Self)
        }
    }

    impl RepositoryWrite for NoOpRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            // nothing to do here
            Ok(())
        }
    }

    struct ErroneousRepoWrite;

    impl ErroneousRepoWrite {
        fn new() -> Box<Self> {
            Box::new(Self)
        }
    }

    impl RepositoryWrite for ErroneousRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            Err(IndexerErr::BusError(BusErr::GenericError(anyhow!("error"))))
        }
    }
}
