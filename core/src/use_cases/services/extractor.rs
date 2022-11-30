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
                    e => trace!("event not supported in TxtExtractor: '{}'", e),
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
    use crate::result::ExtractorErr;
    use crate::testingtools::unit::create_test_shim;
    use crate::testingtools::{pipe, MutexExt, Spy, Tx};

    use anyhow::Result;
    use fake::{Fake, Faker};
    use leptess::tesseract::TessInitError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn extractor_is_used_to_extract_text() -> Result<()> {
        // given
        init_tracing();
        let (spy, extractor) = ExtractorSpy::working();
        let factory_stub = ExtractorFactoryStub::new(vec![extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn data_extracted_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let docs_details: Vec<DocDetails> = Faker.fake();
        let extractor = ExtractorStub::new(docs_details.clone());
        let factory_stub = ExtractorFactoryStub::new(vec![extractor]);
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
        let extractor = ExtractorStub::new(Faker.fake());
        let factory_stub = ExtractorFactoryStub::new(vec![extractor]);
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
        let (spy, failing_extractor) = ExtractorSpy::failing();
        let factory_stub = ExtractorFactoryStub::new(vec![failing_extractor]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_extractor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(spy.method_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn extractor_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let noop_extractor = NoOpExtractor::new();
        let factory_stub = ExtractorFactoryStub::new(vec![noop_extractor]);
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::PipelineFinished,
        ];
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);

        // when
        shim.send_events(&ignored_events)?;

        // then
        assert!(shim.no_such_events(
            &[
                // TODO: those events should not have concrete values inside (any DataExtracted or
                // EncryptionRequest event should cause failure, not only those with concrete values)
                BusEvent::DataExtracted(Vec::new()),
                BusEvent::EncryptDocument(shim.test_location()),
                BusEvent::EncryptThumbnail(shim.test_location())
            ],
            ignored_events.len(),
        )?);
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_extraction_do_not_kill_service() -> Result<()> {
        // given
        let (spy1, failing_extractor1) = ExtractorSpy::failing();
        let (spy2, failing_extractor2) = ExtractorSpy::failing();
        let factory_stub = ExtractorFactoryStub::new(vec![failing_extractor1, failing_extractor2]);
        let mut shim = create_test_shim()?;
        TxtExtractor::new(shim.bus())?.run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor
                                               // let mut publ = bus.publisher();
        shim.trigger_extractor()?;
        assert!(spy1.method_called());

        // when
        shim.trigger_extractor()?;

        // then
        assert!(spy2.method_called());

        Ok(())
    }

    struct ExtractorFactoryStub {
        extractor_stubs: Mutex<Vec<Option<Extractor>>>,
        current: AtomicUsize,
    }

    impl ExtractorFactoryStub {
        // NOTE: this bizzare `Vec` of `Extractor`s is required because every time the extractor is
        // used, it's `take`n from the extractor stub. It has to be taken because it's not possible
        // to extract it from withing a `Mutex` without using `Option`. It has to be inside `Mutex`
        // because it has to be `Sync`, otherwise it won't compile. And finally, it has to be taken
        // because the trait `ExtractorFactory` is supposed to return owned value.
        fn new(extractor_stubs: Vec<Extractor>) -> Box<Self> {
            let extractor_stubs = extractor_stubs.into_iter().map(Option::Some).collect();
            Box::new(Self {
                extractor_stubs: Mutex::new(extractor_stubs),
                current: AtomicUsize::new(0),
            })
        }
    }

    impl ExtractorFactory for ExtractorFactoryStub {
        fn make(&self, _ext: &Ext) -> Extractor {
            let current = self.current.load(Ordering::SeqCst);
            let mut stubs = self.extractor_stubs.lock().expect("poisoned mutex");
            let extractor = stubs[current].take().unwrap();
            self.current.swap(current + 1, Ordering::SeqCst);
            extractor
        }
    }

    struct ExtractorSpy;

    impl ExtractorSpy {
        fn working() -> (Spy, Extractor) {
            let (tx, spy) = pipe();
            (spy, WorkingExtractor::new(tx))
        }

        fn failing() -> (Spy, Extractor) {
            let (tx, spy) = pipe();
            (spy, FailingExtractor::new(tx))
        }
    }

    struct WorkingExtractor {
        tx: Tx,
    }

    impl WorkingExtractor {
        fn new(tx: Tx) -> Box<Self> {
            Box::new(Self { tx })
        }
    }

    impl DataExtractor for WorkingExtractor {
        fn extract_data(
            &self,
            _location: &Location,
        ) -> std::result::Result<Vec<DocDetails>, ExtractorErr> {
            self.tx.signal();
            Ok(Vec::new())
        }
    }

    struct FailingExtractor {
        tx: Tx,
    }

    impl FailingExtractor {
        fn new(tx: Tx) -> Box<Self> {
            Box::new(Self { tx })
        }
    }

    impl DataExtractor for FailingExtractor {
        fn extract_data(
            &self,
            _location: &Location,
        ) -> std::result::Result<Vec<DocDetails>, ExtractorErr> {
            self.tx.signal();
            Err(ExtractorErr::OcrExtract(TessInitError { code: 0 }))
        }
    }

    struct ExtractorStub {
        docs_details: Vec<DocDetails>,
    }

    impl ExtractorStub {
        fn new(docs_details: Vec<DocDetails>) -> Box<Self> {
            Box::new(Self { docs_details })
        }
    }

    impl DataExtractor for ExtractorStub {
        fn extract_data(
            &self,
            _location: &Location,
        ) -> std::result::Result<Vec<DocDetails>, ExtractorErr> {
            // nothing to do
            Ok(self.docs_details.clone())
        }
    }

    struct NoOpExtractor;

    impl NoOpExtractor {
        fn new() -> Box<Self> {
            Box::new(Self)
        }
    }

    impl DataExtractor for NoOpExtractor {
        fn extract_data(&self, _location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
            // nothing to do
            Ok(Vec::new())
        }
    }
}
