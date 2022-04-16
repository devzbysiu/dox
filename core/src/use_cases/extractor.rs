use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;

use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<DocDetails>;
}

pub trait ExtractorFactory {
    fn from_ext(&self, ext: &Ext) -> Extractor;
}

pub type Extractor = Box<dyn TextExtractor>;
