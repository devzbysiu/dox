//! Represents abstractions for extracting text.
use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;

/// Extracts text.
#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    /// Given the [`Location`], extracts text from all documents contained in it.
    fn extract_text(&self, location: &Location) -> Result<Vec<DocDetails>>;
}

/// Creates extractor.
#[allow(clippy::module_name_repetitions)]
pub trait ExtractorFactory: Sync + Send {
    /// Creates different extractors based on the provided extension.
    fn from_ext(&self, ext: &Ext) -> Extractor;
}

pub type Extractor = Box<dyn TextExtractor>;
