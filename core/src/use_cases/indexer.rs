use crate::result::Result;
use crate::use_cases::config::Config;
use crate::use_cases::extractor::ExtractorFactory;
use crate::use_cases::pipe::{ExternalEvent, InternalEvent};
use crate::use_cases::preprocessor::PreprocessorFactory;
use crate::use_cases::repository::Repository;

use eventador::Eventador;
use std::thread;
use tracing::{instrument, warn};

#[allow(unused)]
pub struct Indexer {
    eventbus: Eventador,
    preprocessor_factory: Box<dyn PreprocessorFactory>,
    extractor_factory: Box<dyn ExtractorFactory>,
    repository: Box<dyn Repository>,
}

#[allow(unused)]
impl Indexer {
    pub fn new(
        eventbus: Eventador,
        preprocessor_factory: Box<dyn PreprocessorFactory>,
        extractor_factory: Box<dyn ExtractorFactory>,
        repository: Box<dyn Repository>,
    ) -> Self {
        Self {
            eventbus,
            preprocessor_factory,
            extractor_factory,
            repository,
        }
    }

    #[instrument(skip(self))]
    pub fn run(self, config: Config) {
        thread::spawn(move || -> Result<()> {
            let subscriber = self.eventbus.subscribe::<ExternalEvent>();
            loop {
                match subscriber.recv().to_owned() {
                    ExternalEvent::NewDocs(location) => {
                        let extension = location.extension();
                        let preprocessor = self.preprocessor_factory.from_ext(&extension);
                        let extractor = self.extractor_factory.from_ext(&extension);
                        preprocessor.preprocess(&location, &config.thumbnails_dir)?;
                        self.repository.index(&extractor.extract_text(&location)?)?;
                        self.eventbus.publish(InternalEvent::DocumentReady);
                    }
                }
            }
        });
    }
}
