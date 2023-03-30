use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::result::IndexerErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::state::StateWriter;

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
        // TODO: think about num_threads
        // TODO: should threadpool be shared between services?
        // TODO: should threadpool have it's own abstraction here?
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        Ok(Self { bus, tp })
    }

    #[instrument(skip(self, state))]
    pub fn run(self, state: StateWriter) {
        let sub = self.bus.subscriber();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::DataExtracted(doc_details) => self.index(doc_details, state.clone()),
                    BusEvent::DocumentEncryptionFailed(loc) => self.cleanup(loc, state.clone()),
                    e => trace!("event not supported in indexer: '{:?}'", e),
                }
            }
        });
    }

    #[instrument(skip(self, state))]
    fn index(&self, doc_details: Vec<DocDetails>, state: StateWriter) {
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = index(&doc_details, &state, publ) {
                error!("indexing failed: '{}'", e);
            }
        });
    }

    #[instrument(skip(self, state))]
    fn cleanup(&self, loc: Location, state: StateWriter) {
        debug!("pipeline failed, removing index data");
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = cleanup(&loc, &state, publ) {
                error!("cleanup failed: '{}'", e);
            }
        });
    }
}

#[instrument(skip(state, publ))]
fn index(doc_details: &[DocDetails], state: &StateWriter, publ: EventPublisher) -> Result<()> {
    debug!("start indexing docs");
    state.index(doc_details)?;
    debug!("docs indexed");
    publ.send(BusEvent::Indexed(doc_details.to_vec()))?;
    Ok(())
}

#[instrument(skip(state, publ))]
fn cleanup(loc: &Location, state: &StateWriter, publ: EventPublisher) -> Result<()> {
    state.delete(loc)?;
    publ.send(BusEvent::DataRemoved)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::entities::document::DocDetails;
    use crate::testingtools::services::state::{failing, noop, tracked, working};
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;
    use fake::{Fake, Faker};

    #[test]
    fn state_writer_is_used_to_index_data() -> Result<()> {
        // given
        init_tracing();
        let (state_spies, state) = tracked(&working());
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(state.writer());

        // when
        shim.trigger_indexer(Faker.fake())?;

        // then
        assert!(state_spies.index_called());

        Ok(())
    }

    #[test]
    fn indexed_event_is_send_on_success() -> Result<()> {
        // given
        init_tracing();
        let state = noop();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(state.writer());
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
        let state = noop();
        let mut shim = create_test_shim()?;
        let docs_details: Vec<DocDetails> = Faker.fake();
        Indexer::new(shim.bus())?.run(state.writer());

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
        let state = failing();
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(state.writer());

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
        let state = noop();
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
        Indexer::new(shim.bus())?.run(state.writer());

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
        let (state_spies, state) = tracked(&failing());
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(state.writer());
        shim.trigger_indexer(Faker.fake())?;
        assert!(state_spies.index_called());

        // when
        shim.trigger_indexer(Faker.fake())?;

        // then
        assert!(state_spies.index_called());

        Ok(())
    }

    #[test]
    fn state_removes_data_when_document_encryption_failed_event_appears() -> Result<()> {
        // given
        init_tracing();
        let (state_spies, state) = tracked(&noop());
        let mut shim = create_test_shim()?;
        Indexer::new(shim.bus())?.run(state.writer());

        // when
        shim.trigger_document_encryption_failure()?;

        // then
        assert!(state_spies.delete_called());

        Ok(())
    }
}
