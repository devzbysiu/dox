//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::user::User;

use std::convert::TryFrom;
use std::thread;
use tracing::{debug, instrument, warn};

pub type ExtractorCreator = Box<dyn ExtractorFactory>;

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
                    Event::NewDocs(location) => {
                        debug!("NewDocs in: '{:?}', starting extraction", location);
                        let extension = location.extension();
                        let extractor = extractor_factory.make(&extension);
                        publ.send(Event::TextExtracted(
                            User::try_from(&location)?,
                            extractor.extract_text(&location)?,
                        ))?;
                        debug!("extraction finished");
                        debug!("sending encryption request for: '{:?}'", location);
                        publ.send(Event::EncryptionRequest(location))?;
                    }
                    e => debug!("event not supported in TxtExtractor: {}", e.to_string()),
                }
            }
        });
    }
}

/// Extracts text.
pub trait TextExtractor {
    /// Given the [`Location`], extracts text from all documents contained in it.
    fn extract_text(&self, location: &Location) -> Result<Vec<DocDetails>>;
}

/// Creates extractor.
pub trait ExtractorFactory: Sync + Send {
    /// Creates different extractors based on the provided extension.
    fn make(&self, ext: &Ext) -> Extractor;
}

pub type Extractor = Box<dyn TextExtractor>;
