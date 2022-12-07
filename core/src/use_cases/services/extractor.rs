//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::ExtractorErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::thread;
use tracing::{debug, error, instrument, trace, warn};

pub type ExtractorCreator = Box<dyn ExtractorFactory>;
pub type Extractor = Box<dyn DataExtractor>;
type Result<T> = std::result::Result<T, ExtractorErr>;

pub struct TxtExtractor {
    bus: EventBus,
    tp: ThreadPool,
}

impl TxtExtractor {
    pub fn new(bus: EventBus) -> Result<Self> {
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        Ok(Self { bus, tp })
    }

    #[instrument(skip(self, factory))]
    pub fn run(self, factory: ExtractorCreator) {
        thread::spawn(move || -> Result<()> {
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    BusEvent::DocsMoved(loc) => self.extract_data(loc, &factory)?,
                    e => trace!("event not supported in TxtExtractor: '{:?}'", e),
                }
            }
        });
    }

    fn extract_data(&self, loc: Location, factory: &ExtractorCreator) -> Result<()> {
        debug!("NewDocs in: '{:?}', starting extraction", loc);
        let extractor = factory.make(&loc.extension()?);
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = extract(loc, &extractor, publ) {
                error!("extraction failed: '{}'", e);
            }
        });
        Ok(())
    }
}

fn extract(loc: Location, extr: &Extractor, mut publ: EventPublisher) -> Result<()> {
    publ.send(BusEvent::DataExtracted(extr.extract_data(&loc)?))?;
    debug!("extraction finished");
    debug!("sending encryption request for: '{:?}'", loc);
    publ.send(BusEvent::EncryptDocument(loc))?;
    Ok(())
}

/// Extracts text.
pub trait DataExtractor: Send {
    /// Given the [`Location`], extracts text from all documents contained in it.
    fn extract_data(&self, location: &Location) -> Result<Vec<DocDetails>>;
}

/// Creates extractor.
pub trait ExtractorFactory: Sync + Send {
    /// Creates different extractors based on the provided extension.
    fn make(&self, ext: &Ext) -> Extractor;
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::services::extractor::{
        factory, failing, noop, stub, tracked, working,
    };
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::time::Duration;

    #[test]
    fn extractor_is_used_to_extract_text() -> Result<()> {
        // given
        init_tracing();
        let (extractor_spies, extractor) = tracked(working());
        let factory_stub = factory(vec![extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;

        // then
        assert!(extractor_spies.extract_called());

        Ok(())
    }

    #[test]
    fn data_extracted_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let docs_details: Vec<DocDetails> = Faker.fake();
        let extractor = stub(docs_details.clone());
        let factory_stub = factory(vec![extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;
        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(shim.event_on_bus(&BusEvent::DataExtracted(docs_details))?);

        Ok(())
    }

    #[test]
    fn encrypt_document_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let extractor = stub(Faker.fake());
        let factory_stub = factory(vec![extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;

        shim.ignore_event()?; // ignore NewDocs event
        shim.ignore_event()?; // ignore TextExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::EncryptDocument(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_extractor_fails() -> Result<()> {
        // given
        init_tracing();
        let (extractor_spies, extractor) = tracked(failing());
        let factory_stub = factory(vec![extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(extractor_spies.extract_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn extractor_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let noop_extractor = noop();
        let factory_stub = factory(vec![noop_extractor]);
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::EncryptThumbnail(Faker.fake()),
            BusEvent::DocumentEncryptionFailed(Faker.fake()),
            BusEvent::ThumbnailEncryptionFailed(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::ThumbnailRemoved,
            BusEvent::PipelineFinished,
        ];
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);

        // when
        shim.send_events(&ignored_events)?;

        // then
        // no DataExtracted and EncryptDocument on the bus
        for _ in 0..ignored_events.len() {
            let received_event = shim.recv_event()?;
            assert!(!matches!(received_event, BusEvent::DataExtracted(_)));
            assert!(!matches!(received_event, BusEvent::EncryptDocument(_)));
        }
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_extraction_do_not_kill_service() -> Result<()> {
        // given
        let (extractor_spies1, extractor1) = tracked(failing());
        let (extractor_spies2, extractor2) = tracked(failing());
        let factory_stub = factory(vec![extractor1, extractor2]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        shim.trigger_extractor()?;
        assert!(extractor_spies1.extract_called());

        // when
        shim.trigger_extractor()?;

        // then
        assert!(extractor_spies2.extract_called());

        Ok(())
    }
}
