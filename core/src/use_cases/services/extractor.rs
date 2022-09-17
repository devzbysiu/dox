//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::user::User;

use std::convert::TryFrom;
use std::thread;
use tracing::{debug, instrument, warn};

pub type ExtractorCreator = Box<dyn ExtractorFactory>;
pub type Extractor = Box<dyn TextExtractor>;

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
                        publ.send(BusEvent::TextExtracted(
                            User::try_from(&location)?,
                            extractor.extract_text(&location)?,
                        ))?;
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
pub trait TextExtractor: Send {
    /// Given the [`Location`], extracts text from all documents contained in it.
    fn extract_text(&self, location: &Location) -> Result<Vec<DocDetails>>;
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

    use anyhow::Result;
    use std::fs::{self, create_dir_all};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::Mutex;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn extractor_is_used_to_extract_text() -> Result<()> {
        // given
        init_tracing();
        let (spy, extractor) = ExtractorSpy::create();
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let tmp_dir = tempdir()?;
        let user_dir_name = base64::encode("some@email.com");
        let user_dir = tmp_dir.path().join(user_dir_name);
        create_dir_all(&user_dir)?;
        let new_file_path = user_dir.join("some-file.jpg");
        fs::write(&new_file_path, "anything")?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);
        let mut publ = bus.publisher();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file_path.into()])))?;

        // then
        assert!(spy.extract_called());

        Ok(())
    }

    #[test]
    fn text_extracted_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let docs_details = vec![DocDetails::new("path", "body", "thumbnail")];
        let extractor = Box::new(ExtractorStub::new(docs_details.clone()));
        let factory_stub = Box::new(ExtractorFactoryStub::new(extractor));
        let tmp_dir = tempdir()?;
        let user_dir_name = base64::encode("some@email.com");
        let user_dir = tmp_dir.path().join(user_dir_name);
        create_dir_all(&user_dir)?;
        let new_file_path = user_dir.join("some-file.jpg");
        fs::write(&new_file_path, "anything")?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file_path.into()])))?;

        let _event = sub.recv()?; // ignore NewDocs event

        // then
        if let BusEvent::TextExtracted(user, details) = sub.recv()? {
            assert_eq!(user, User::new("some@email.com"));
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
        let tmp_dir = tempdir()?;
        let user_dir_name = base64::encode("some@email.com");
        let user_dir = tmp_dir.path().join(user_dir_name);
        create_dir_all(&user_dir)?;
        let new_file_path = user_dir.join("some-file.jpg");
        fs::write(&new_file_path, "anything")?;
        let bus = Box::new(LocalBus::new()?);

        // when
        let txt_extractor = TxtExtractor::new(&bus);
        txt_extractor.run(factory_stub);

        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::NewDocs(Location::FS(vec![new_file_path
            .clone()
            .into()])))?;

        let _event = sub.recv()?; // ignore NewDocs event
        let _event = sub.recv()?; // ignore TextExtracted event

        // then
        if let BusEvent::EncryptionRequest(location) = sub.recv()? {
            assert_eq!(location, Location::FS(vec![new_file_path.into()]));
        } else {
            panic!("invalid event appeared");
        }

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

    impl TextExtractor for ExtractorSpy {
        fn extract_text(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
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

    impl TextExtractor for ExtractorStub {
        fn extract_text(&self, _location: &Location) -> crate::result::Result<Vec<DocDetails>> {
            // nothing to do
            Ok(self.docs_details.clone())
        }
    }
}
