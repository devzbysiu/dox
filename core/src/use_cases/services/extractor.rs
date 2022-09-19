//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};

use std::thread;
use tracing::{debug, instrument, warn};

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
    pub fn run(&self, extractor_factory: ExtractorCreator) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::NewDocs(location) => {
                        debug!("NewDocs in: '{:?}', starting extraction", location);
                        let extension = location.extension();
                        let extractor = extractor_factory.make(&extension);
                        publ.send(BusEvent::DataExtracted(extractor.extract_data(&location)?))?;
                        debug!("extraction finished");
                        debug!("sending encryption request for: '{:?}'", location);
                        publ.send(BusEvent::EncryptionRequest(location))?;
                    }
                    e => debug!("event not supported in TxtExtractor: {}", e.to_string()),
                }
            }
        });
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
    use crate::testutils::{mk_file, SubscriberExt};
    use crate::use_cases::user::User;

    use anyhow::Result;
    use leptess::tesseract::TessInitError;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn extractor_is_used_to_extract_text() -> Result<()> {
        // given
        init_tracing();
        let (spy, extractor) = ExtractorSpy::create();
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);
        let mut publ = bus.publisher();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

        // then
        assert!(spy.extract_called());

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
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);

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
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);

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
        let extractor = Box::new(ErroneousExtractor);
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let new_file = mk_file(base64::encode("some@email.com"), "some-file.jpg".into())?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file.path])))?;

        let _event = sub.recv()?; // ignore NewDocs event

        // then
        assert!(sub.try_recv(Duration::from_secs(2)).is_err()); // no more events on the bus

        Ok(())
    }

    struct ExtractorFactoryStub {
        extractor_stub: Mutex<Option<Extractor>>,
    }

    impl ExtractorFactoryStub {
        fn new(extractor_stub: Extractor) -> Self {
            Self {
                extractor_stub: Mutex::new(Some(extractor_stub)),
            }
        }
    }

    impl ExtractorFactory for ExtractorFactoryStub {
        fn make(&self, _ext: &Ext) -> Extractor {
            self.extractor_stub
                .lock()
                .expect("poisoned mutex")
                .take()
                .unwrap()
        }
    }

    struct ExtractorSpy {
        tx: Mutex<Sender<()>>,
    }

    impl ExtractorSpy {
        fn create() -> (Spy, Extractor) {
            let (tx, rx) = channel();
            (Spy::new(rx), Box::new(Self { tx: Mutex::new(tx) }))
        }
    }

    impl DataExtractor for ExtractorSpy {
        fn extract_data(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(Vec::new())
        }
    }

    struct Spy {
        rx: Receiver<()>,
    }

    impl Spy {
        fn new(rx: Receiver<()>) -> Self {
            Self { rx }
        }

        fn extract_called(&self) -> bool {
            self.rx.recv_timeout(Duration::from_secs(2)).is_ok()
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

    struct ErroneousExtractor;

    impl DataExtractor for ErroneousExtractor {
        fn extract_data(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            Err(DoxErr::OcrExtract(TessInitError { code: 0 }))
        }
    }
}
