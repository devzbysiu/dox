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

/// Generates thumbnail either for PDF file or image file when [`Event::NewDocs`] appears on the
/// bus.
///
/// Depending on the [`Location::extension`], specific preprocessor is selected (see
/// [`FilePreprocessor`]). It then calls [`FilePreprocessor::preprocess`] method.
pub struct ThumbnailGenerator<'a> {
    cfg: &'a Config,
    bus: &'a dyn Bus,
}

impl<'a> ThumbnailGenerator<'a> {
    pub fn new(cfg: &'a Config, bus: &'a dyn Bus) -> Self {
        Self { cfg, bus }
    }

    #[instrument(skip(self, preprocessor_factory))]
    pub fn run(&self, preprocessor_factory: Box<dyn PreprocessorFactory>) {
        let thumbnails_dir = self.cfg.thumbnails_dir.clone();
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
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
