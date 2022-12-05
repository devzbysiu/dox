use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::result::IndexerErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::repository::RepoWrite;

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::thread;
use tracing::{debug, error, instrument, trace, warn};
type Result<T> = std::result::Result<T, IndexerErr>;

pub struct Indexer {
    bus: EventBus,
    tp: ThreadPool,
}

impl Indexer {
    pub fn new(bus: EventBus) -> Result<Self> {
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        Ok(Self { bus, tp })
    }

    #[instrument(skip(self, repo))]
    pub fn run(self, repo: RepoWrite) {
        // TODO: think about num_threads
        // TODO: should threadpool be shared between services?
        // TODO: should threadpool have it's own abstraction here?
        let sub = self.bus.subscriber();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::DataExtracted(doc_details) => self.index(doc_details, repo.clone()),
                    BusEvent::DocumentEncryptionFailed(loc) => self.cleanup(loc, repo.clone()),
                    e => trace!("event not supported in indexer: '{:?}'", e),
                }
            }
        });
    }

    #[instrument(skip(self, repo))]
    fn index(&self, doc_details: Vec<DocDetails>, repo: RepoWrite) {
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = index(&doc_details, &repo, publ) {
                error!("indexing failed: '{}'", e);
            }
        });
    }

    #[instrument(skip(self, repo))]
    fn cleanup(&self, loc: Location, repo: RepoWrite) {
        debug!("pipeline failed, removing index data");
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = cleanup(&loc, &repo, publ) {
                error!("cleanup failed: '{}'", e);
            }
        });
    }
}

#[instrument(skip(repo, publ))]
fn index(doc_details: &[DocDetails], repo: &RepoWrite, mut publ: EventPublisher) -> Result<()> {
    debug!("start indexing docs");
    repo.index(doc_details)?;
    debug!("docs indexed");
    publ.send(BusEvent::Indexed(doc_details.to_vec()))?;
    Ok(())
}

#[instrument(skip(repo, publ))]
fn cleanup(loc: &Location, repo: &RepoWrite, mut publ: EventPublisher) -> Result<()> {
    repo.delete(loc)?;
    publ.send(BusEvent::DataRemoved)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::entities::document::DocDetails;
    use crate::entities::user::User;
    use crate::result::{BusErr, IndexerErr, SearchErr};
    use crate::testingtools::services::indexer::tracked_repo;
    use crate::testingtools::unit::create_test_shim;
    use crate::testingtools::{pipe, MutexExt, Spy, Tx};
    use crate::use_cases::repository::{
        Repo, RepoRead, Repository, RepositoryRead, RepositoryWrite, SearchResult,
    };

    use anyhow::{anyhow, Result};
    use fake::{Fake, Faker};
    use std::sync::Arc;

    #[test]
    fn repo_write_is_used_to_index_data() -> Result<()> {
        // given
        init_tracing();
        let (spy, working_repo_write) = RepoWriteSpy::working();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(working_repo_write);

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
        Indexer::new(shim.bus())?.run(noop_repo_write);
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
        Indexer::new(shim.bus())?.run(repo_write);

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
        Indexer::new(shim.bus())?.run(repo_write);

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
            BusEvent::DocsMoved(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::EncryptDocument(Faker.fake()),
            BusEvent::EncryptThumbnail(Faker.fake()),
            BusEvent::ThumbnailEncryptionFailed(Faker.fake()),
            BusEvent::ThumbnailRemoved,
            BusEvent::PipelineFinished,
        ];
        Indexer::new(shim.bus())?.run(noop_repo_write);

        // when
        shim.send_events(&ignored_events)?;

        // then
        // no Indexed and DataRemoved emitted
        for _ in 0..ignored_events.len() {
            let received = shim.recv_event()?;
            assert!(!matches!(received, BusEvent::Indexed(_)));
            assert!(!matches!(received, BusEvent::DataRemoved));
        }
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_indexing_do_not_kill_service() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_repo_write) = RepoWriteSpy::failing();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(failing_repo_write);
        shim.trigger_indexer(Faker.fake())?;
        assert!(spy.method_called());

        // when
        shim.trigger_indexer(Faker.fake())?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn repo_removes_data_when_document_encryption_failed_event_appears() -> Result<()> {
        // given
        init_tracing();
        let (repo_spies, repo) = tracked_repo(&noop_repo());
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(repo.write());

        // when
        shim.trigger_document_encryption_failure()?;

        // then
        assert!(repo_spies.delete_called());

        Ok(())
    }

    struct RepoWriteSpy;

    impl RepoWriteSpy {
        fn working() -> (Spy, RepoWrite) {
            let (tx, spy) = pipe();
            (spy, WorkingRepoWrite::make(tx))
        }

        fn failing() -> (Spy, RepoWrite) {
            let (tx, spy) = pipe();
            (spy, FailingRepoWrite::make(tx))
        }
    }

    struct WorkingRepoWrite {
        tx: Tx,
    }

    impl WorkingRepoWrite {
        fn make(tx: Tx) -> Arc<Self> {
            Arc::new(Self { tx })
        }
    }

    impl RepositoryWrite for WorkingRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            self.tx.signal();
            Ok(())
        }

        fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
            unimplemented!()
        }
    }

    struct FailingRepoWrite {
        tx: Tx,
    }

    impl FailingRepoWrite {
        fn make(tx: Tx) -> Arc<Self> {
            Arc::new(Self { tx })
        }
    }

    impl RepositoryWrite for FailingRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            self.tx.signal();
            Err(IndexerErr::Bus(BusErr::Generic(anyhow!("error"))))
        }

        fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
            unimplemented!()
        }
    }

    fn noop_repo() -> Repo {
        NoOpRepo::make()
    }

    struct NoOpRepo {
        read: RepoRead,
        write: RepoWrite,
    }

    impl NoOpRepo {
        fn make() -> Repo {
            Box::new(Self {
                read: NoOpRepoRead::new(),
                write: NoOpRepoWrite::new(),
            })
        }
    }

    impl Repository for NoOpRepo {
        fn read(&self) -> RepoRead {
            self.read.clone()
        }

        fn write(&self) -> RepoWrite {
            self.write.clone()
        }
    }

    struct NoOpRepoWrite;

    impl NoOpRepoWrite {
        fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl RepositoryWrite for NoOpRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            // nothing to do here
            Ok(())
        }

        fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
            // nothing to do here
            Ok(())
        }
    }

    struct NoOpRepoRead;

    impl NoOpRepoRead {
        fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl RepositoryRead for NoOpRepoRead {
        fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
            // nothing to do
            Ok(Vec::new().into())
        }

        fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
            // nothing to do
            Ok(Vec::new().into())
        }
    }

    struct ErroneousRepoWrite;

    impl ErroneousRepoWrite {
        fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl RepositoryWrite for ErroneousRepoWrite {
        fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
            Err(IndexerErr::Bus(BusErr::Generic(anyhow!("error"))))
        }

        fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
            unimplemented!()
        }
    }
}
