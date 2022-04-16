use crate::cfg::Config;
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::entities::location::Location;
use crate::indexer::{self, RepoTools};
use crate::notifier::new_doc_notifier;
use crate::result::Result;
use crate::use_cases::event::{Event, Sink};
use crate::use_cases::extractor::ExtractorFactory;
use crate::use_cases::notifier::Notifier;
use crate::use_cases::preprocessor::PreprocessorFactory;

use std::thread;
use tracing::{debug, error, instrument, warn};

pub struct Indexer {
    sink: Box<dyn Sink>,
    notifier: Box<dyn Notifier>,
    preprocessor_factory: Box<dyn PreprocessorFactory>,
    extractor_factory: Box<dyn ExtractorFactory>,
    repository: Box<dyn Repository>,
}

impl Indexer {
    fn new(
        sink: Box<dyn Sink>,
        notifier: Box<dyn Notifier>,
        preprocessor_factory: Box<dyn PreprocessorFactory>,
        extractor_factory: Box<dyn ExtractorFactory>,
    ) -> Self {
        Self {
            sink,
            notifier,
            preprocessor_factory,
            extractor_factory,
        }
    }

    #[instrument(skip(self))]
    fn run(self) {
        thread::spawn(move || -> Result<()> {
            loop {
                match self.sink.recv() {
                    Ok(Event::NewDocs(location)) => {
                        let extension = location.extension();
                        let preprocessor = self.preprocessor_factory.from_ext(&extension);
                        let extractor = self.extractor_factory.from_ext(&extension);
                        preprocessor.preprocess_location(&location)?;
                        let tuples = extractor.extract(&location)?;
                        self.repository.index(tuples);
                        self.notifier.notify()?;
                    }
                    Err(e) => error!("failed to receive event: {}", e),
                }
            }
        });
    }
}
