use crate::extractor::{Extractor, FilenameToBody};

#[derive(Debug, Default)]
pub struct Pdf;

impl Extractor for Pdf {
    fn extract_text(&self, _paths: &[std::path::PathBuf]) -> Vec<FilenameToBody> {
        vec![]
    }
}
