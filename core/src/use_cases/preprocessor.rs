//! Abstraction for preprocessing received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::config::Config;

use std::path::Path;
use std::thread;
use tracing::log::debug;
use tracing::{instrument, warn};

pub struct ThumbnailGenerator;

impl ThumbnailGenerator {
    #[instrument(skip(bus, preprocessor_factory))]
    pub fn run(cfg: &Config, bus: &dyn Bus, preprocessor_factory: Box<dyn PreprocessorFactory>) {
        let thumbnails_dir = cfg.thumbnails_dir.clone();
        let sub = bus.subscriber();
        let mut publ = bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Event::NewDocs(location) = sub.recv()? {
                    let extension = location.extension();
                    let preprocessor = preprocessor_factory.make(&extension);
                    preprocessor.preprocess(&location, &thumbnails_dir)?;
                    publ.send(Event::ThumbnailMade)?;
                } else {
                    debug!("event not supported here");
                }
            }
        });
    }
}

/// Abstracts the process of preprocessing received document.
///
/// This happens right after the document was received. See
/// [`Indexer::run`](crate::use_cases::indexer::Indexer::run).
pub trait FilePreprocessor {
    fn preprocess(&self, location: &Location, thumbnails_dir: &Path) -> Result<()>;
}

/// Creates [`Preprocessor`].
pub trait PreprocessorFactory: Sync + Send {
    /// Creates [`Preprocessor`] based on the extesion. PDF files require different preprocessing
    /// than images.
    fn make(&self, ext: &Ext) -> Preprocessor;
}

pub type Preprocessor = Box<dyn FilePreprocessor>;
