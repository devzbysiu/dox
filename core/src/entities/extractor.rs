use super::document::DocDetails;

use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<DocDetails>;
}

pub type Extractor = Box<dyn TextExtractor>;
