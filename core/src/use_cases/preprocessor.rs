//! Abstraction for preprocessing received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::config::Config;

use std::path::PathBuf;

/// Abstracts the process of preprocessing received document.
///
/// This happens right after the document was received. See
/// [`Indexer::run`](crate::use_cases::indexer::Indexer::run).
#[allow(clippy::module_name_repetitions)]
pub trait FilePreprocessor {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()>;
    fn preprocess_location(&self, location: &Location) -> Result<()>;
}

/// Creates [`Preprocessor`].
#[allow(clippy::module_name_repetitions)]
pub trait PreprocessorFactory: Sync + Send {
    /// Creates [`Preprocessor`] based on the extesion. PDF files require different preprocessing
    /// than images.
    fn from_ext(&self, ext: &Ext, config: &Config) -> Preprocessor;
}

pub type Preprocessor = Box<dyn FilePreprocessor>;
