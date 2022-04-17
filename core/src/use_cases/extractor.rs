use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;

use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<DocDetails>;
    fn extract_text_from_location(&self, location: &Location) -> Result<Vec<DocDetails>>;
}

#[allow(clippy::module_name_repetitions)]
pub trait ExtractorFactory: Sync + Send {
    fn from_ext(&self, ext: &Ext) -> Extractor;
}

pub type Extractor = Box<dyn TextExtractor>;
