//! Abstraction for preprocessing received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::PreprocessorErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::config::Config;

use rayon::ThreadPoolBuilder;
use std::path::Path;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub type PreprocessorCreator = Box<dyn PreprocessorFactory>;
pub type Preprocessor = Box<dyn FilePreprocessor>;

/// Generates thumbnail either for PDF file or image file when [`Event::NewDocs`] appears on the
/// bus.
///
/// Depending on the [`Location::extension`], specific preprocessor is selected (see
/// [`FilePreprocessor`]). It then calls [`FilePreprocessor::preprocess`] method.
pub struct ThumbnailGenerator {
    cfg: Config,
    bus: EventBus,
}

impl ThumbnailGenerator {
    pub fn new(cfg: Config, bus: EventBus) -> Self {
        Self { cfg, bus }
    }

    #[instrument(skip(self, preprocessor_factory))]
    pub fn run(self, preprocessor_factory: PreprocessorCreator) {
        thread::spawn(move || -> Result<(), PreprocessorErr> {
            let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
            let thumbnails_dir = self.cfg.thumbnails_dir.clone();
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    BusEvent::NewDocs(loc) => {
                        debug!("NewDocs in: '{:?}', starting preprocessing", loc);
                        let extension = loc.extension();
                        let preprocessor = preprocessor_factory.make(&extension);
                        let publ = self.bus.publisher();
                        let dir = thumbnails_dir.clone();
                        tp.spawn(move || {
                            if let Err(e) = preprocess(&loc, &preprocessor, &dir, publ) {
                                error!("extraction failed: '{}'", e);
                            }
                        });
                    }
                    e => debug!("event not supported in ThumbnailGenerator: '{}'", e),
                }
            }
        });
    }
}

fn preprocess<P: AsRef<Path>>(
    loc: &Location,
    prepr: &Preprocessor,
    thumbnails_dir: P,
    mut publ: EventPublisher,
) -> Result<(), PreprocessorErr> {
    let thumbnails_dir = thumbnails_dir.as_ref();
    let thumbnail_loc = prepr.preprocess(loc, &thumbnails_dir)?;
    debug!("preprocessing finished");
    publ.send(BusEvent::ThumbnailMade(thumbnail_loc.clone()))?;
    debug!("sending encryption request for: '{:?}'", thumbnail_loc);
    publ.send(BusEvent::EncryptionRequest(thumbnail_loc))?;
    Ok(())
}

/// Abstracts the process of preprocessing received document.
///
/// This happens right after the document was received. See
/// [`Indexer::run`](crate::use_cases::indexer::Indexer::run).
pub trait FilePreprocessor: Send {
    /// Take source location as the input and the parent directory for the output.
    /// Returns the final location of the preprocessing.
    fn preprocess(
        &self,
        location: &Location,
        thumbnails_dir: &Path,
    ) -> Result<Location, PreprocessorErr>;
}

/// Creates [`Preprocessor`].
pub trait PreprocessorFactory: Sync + Send {
    /// Creates [`Preprocessor`] based on the extesion. PDF files require different preprocessing
    /// than images.
    fn make(&self, ext: &Ext) -> Preprocessor;
}
