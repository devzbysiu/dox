use crate::configuration::cfg::Config;
use crate::result::Result;
use crate::use_cases::event::{Event, Input};
use crate::use_cases::extractor::ExtractorFactory;
use crate::use_cases::notifier::Notifier;
use crate::use_cases::preprocessor::PreprocessorFactory;
use crate::use_cases::repository::Repository;

use std::thread;
use tracing::{error, instrument, warn};

#[allow(unused)]
pub struct Indexer {
    sink: Box<dyn Input>,
    notifier: Box<dyn Notifier>,
    preprocessor_factory: Box<dyn PreprocessorFactory>,
    extractor_factory: Box<dyn ExtractorFactory>,
    repository: Box<dyn Repository>,
}

#[allow(unused)]
impl Indexer {
    pub fn new(
        sink: Box<dyn Input>,
        notifier: Box<dyn Notifier>,
        preprocessor_factory: Box<dyn PreprocessorFactory>,
        extractor_factory: Box<dyn ExtractorFactory>,
        repository: Box<dyn Repository>,
    ) -> Self {
        Self {
            sink,
            notifier,
            preprocessor_factory,
            extractor_factory,
            repository,
        }
    }

    #[instrument(skip(self))]
    pub fn run(self, config: Config) {
        thread::spawn(move || -> Result<()> {
            loop {
                match self.sink.recv() {
                    Ok(Event::NewDocs(location)) => {
                        let extension = location.extension();
                        let preprocessor = self.preprocessor_factory.from_ext(&extension, &config);
                        let extractor = self.extractor_factory.from_ext(&extension);
                        preprocessor.preprocess_location(&location)?;
                        let tuples = extractor.extract_text_from_location(&location)?;
                        self.repository.index(tuples)?;
                        self.notifier.notify()?;
                    }
                    Err(e) => error!("failed to receive event: {}", e),
                }
            }
        });
    }
}
