use crate::extractor::{FilenameToBody, TextExtractor};
use crate::result::Result;

use log::debug;
use pdf_extract::extract_text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::Path;

#[derive(Debug, Default)]
pub struct Pdf;

impl TextExtractor for Pdf {
    fn extract_text(&self, paths: &[std::path::PathBuf]) -> Vec<FilenameToBody> {
        debug!("extracting text from pdf...");
        paths
            .par_iter()
            .map(extract)
            .filter_map(Result::ok)
            .collect::<Vec<FilenameToBody>>()
    }
}

fn extract<P: AsRef<Path>>(path: P) -> Result<FilenameToBody> {
    let path = path.as_ref();
    debug!("extracting text from PDF on {}", path.display());
    Ok(FilenameToBody::new(&path, extract_text(path)?))
}
