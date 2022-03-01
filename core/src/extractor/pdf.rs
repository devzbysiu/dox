use crate::extractor::{DocDetails, TextExtractor};
use crate::helpers::PathRefExt;
use crate::result::Result;

use log::debug;
use pdf_extract::extract_text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::Path;

#[derive(Debug, Default)]
pub struct Pdf;

impl TextExtractor for Pdf {
    fn extract_text(&self, paths: &[std::path::PathBuf]) -> Vec<DocDetails> {
        debug!("extracting text from pdf...");
        paths
            .par_iter()
            .map(extract)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>()
    }
}

fn extract<P: AsRef<Path>>(path: P) -> Result<DocDetails> {
    let path = path.as_ref();
    debug!("extracting text from PDF on {}", path.display());
    Ok(DocDetails::new(path, extract_text(path)?, thumbnail(path)))
}

fn thumbnail<P: AsRef<Path>>(path: P) -> String {
    format!("{}.png", path.filestem())
}
