//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};

use rayon::ThreadPoolBuilder;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub type ExtractorCreator = Box<dyn ExtractorFactory>;
pub type Extractor = Box<dyn DataExtractor>;

pub struct TxtExtractor<'a> {
    bus: &'a dyn Bus,
}

impl<'a> TxtExtractor<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, extractor_factory))]
    pub fn run(&self, extractor_factory: ExtractorCreator) -> Result<()> {
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::NewDocs(location) => {
                        if let Err(e) = tp.install(|| -> Result<()> {
                            debug!("NewDocs in: '{:?}', starting extraction", location);
                            let extension = location.extension();
                            let extractor = extractor_factory.make(&extension);
                            publ.send(BusEvent::DataExtracted(extractor.extract_data(&location)?))?;
                            debug!("extraction finished");
                            debug!("sending encryption request for: '{:?}'", location);
                            publ.send(BusEvent::EncryptionRequest(location))?;
                            Ok(())
                        }) {
                            error!("extraction failed: '{}'", e);
                        }
                    }
                    e => debug!("event not supported in TxtExtractor: '{}'", e),
                }
            }
        });
        Ok(())
    }
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
    use crate::data_providers::bus::LocalBus;
    use crate::result::DoxErr;
    use crate::testutils::{mk_file, Spy, SubscriberExt};
    use crate::use_cases::user::User;

    use anyhow::Result;
    use leptess::tesseract::TessInitError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn extractor_is_used_to_extract_text() -> Result<()> {
        // given
        init_tracing();
        let (spy, extractor) = ExtractorSpy::working();
        let factory_stub = Box::new(ExtractorFactoryStub::new(vec![extractor]));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub)?;
        let mut publ = bus.publisher();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn data_extracted_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let docs_details = vec![DocDetails::new(
            User::new("some@email.com"),
            "path",
            "body",
            "thumbnail",
        )];
        let extractor = Box::new(ExtractorStub::new(docs_details.clone()));
        let factory_stub = Box::new(ExtractorFactoryStub::new(vec![extractor]));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub)?;

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

        let _event = sub.recv()?; // ignore NewDocs event

        // then
        if let BusEvent::DataExtracted(details) = sub.recv()? {
            assert_eq!(details, docs_details);
        } else {
            panic!("invalid event appeared");
        }

        Ok(())
    }

    #[test]
    fn encryption_request_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let extractor = Box::new(ExtractorStub::new(Vec::new()));
        let factory_stub = Box::new(ExtractorFactoryStub::new(vec![extractor]));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub)?;

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path.clone()])))?;

        let _event = sub.recv()?; // ignore NewDocs event
        let _event = sub.recv()?; // ignore TextExtracted event

        // then
        if let BusEvent::EncryptionRequest(location) = sub.recv()? {
            assert_eq!(location, Location::FS(vec![new_file.path]));
        } else {
            panic!("invalid event appeared");
        }

        Ok(())
    }

    #[test]
    fn no_event_appears_when_extractor_fails() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_extractor) = ExtractorSpy::failing();
        let factory_stub = Box::new(ExtractorFactoryStub::new(vec![failing_extractor]));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub)?;

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

        let _event = sub.recv()?; // ignore NewDocs event

        // then
        assert!(spy.method_called());
        assert!(sub.try_recv(Duration::from_secs(2)).is_err()); // no more events on the bus

        Ok(())
    }

    #[test]
    fn failure_during_extraction_do_not_kill_service() -> Result<()> {
        // given
        let (spy1, failing_extractor1) = ExtractorSpy::failing();
        let (spy2, failing_extractor2) = ExtractorSpy::failing();
        let factory_stub = Box::new(ExtractorFactoryStub::new(vec![
            failing_extractor1,
            failing_extractor2,
        ]));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = LocalBus::new()?;

        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub)?;
        let mut publ = bus.publisher();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path.clone()])))?;
        assert!(spy1.method_called());

        // when
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

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
        fn new(extractor_stubs: Vec<Extractor>) -> Self {
            let extractor_stubs = extractor_stubs.into_iter().map(Option::Some).collect();
            Self {
                extractor_stubs: Mutex::new(extractor_stubs),
                current: AtomicUsize::new(0),
            }
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
            let (tx, rx) = channel();
            (Spy::new(rx), WorkingExtractor::new(tx))
        }

        fn failing() -> (Spy, Extractor) {
            let (tx, rx) = channel();
            (Spy::new(rx), FailingExtractor::new(tx))
        }
    }

    struct WorkingExtractor {
        tx: Mutex<Sender<()>>,
    }

    impl WorkingExtractor {
        fn new(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl DataExtractor for WorkingExtractor {
        fn extract_data(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(Vec::new())
        }
    }

    struct FailingExtractor {
        tx: Mutex<Sender<()>>,
    }

    impl FailingExtractor {
        fn new(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl DataExtractor for FailingExtractor {
        fn extract_data(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Err(DoxErr::OcrExtract(TessInitError { code: 0 }))
        }
    }

    struct ExtractorStub {
        docs_details: Vec<DocDetails>,
    }

    impl ExtractorStub {
        fn new(docs_details: Vec<DocDetails>) -> Self {
            Self { docs_details }
        }
    }

    impl DataExtractor for ExtractorStub {
        fn extract_data(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            // nothing to do
            Ok(self.docs_details.clone())
        }
    }
}
