use crate::result::Result;
use crate::use_cases::bus::{Bus, Event, ExternalEvent, InternalEvent};
use crate::use_cases::config::Config;
use crate::use_cases::extractor::ExtractorFactory;
use crate::use_cases::preprocessor::PreprocessorFactory;
use crate::use_cases::repository::Repository;

use std::thread;
use tracing::log::debug;
use tracing::{instrument, warn};

pub struct Indexer {
    bus: Box<dyn Bus>,
    preprocessor_factory: Box<dyn PreprocessorFactory>,
    extractor_factory: Box<dyn ExtractorFactory>,
    repository: Box<dyn Repository>,
}

impl Indexer {
    pub fn new(
        bus: Box<dyn Bus>,
        preprocessor_factory: Box<dyn PreprocessorFactory>,
        extractor_factory: Box<dyn ExtractorFactory>,
        repository: Box<dyn Repository>,
    ) -> Self {
        Self {
            bus,
            preprocessor_factory,
            extractor_factory,
            repository,
        }
    }

    #[instrument(skip(self))]
    pub fn run(self, config: Config) {
        thread::spawn(move || -> Result<()> {
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    Event::External(ExternalEvent::NewDocs(location)) => {
                        let extension = location.extension();
                        let preprocessor = self.preprocessor_factory.from_ext(&extension);
                        let extractor = self.extractor_factory.from_ext(&extension);
                        preprocessor.preprocess(&location, &config.thumbnails_dir)?;
                        self.repository.index(&extractor.extract_text(&location)?)?;
                        self.bus
                            .send(Event::Internal(InternalEvent::DocumentReady))?;
                    }
                    _ => debug!("event not supported here"),
                }
            }
        });
    }
}
