use crate::extractor::{DocDetails, TextExtractor};
use crate::result::Result;

use leptess::LepTess;
use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Ocr;

impl TextExtractor for Ocr {
    fn extract_text(&self, paths: &[PathBuf]) -> Vec<DocDetails> {
        debug!("extracting text from image...");
        paths
            .par_iter()
            .map(do_ocr)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>()
    }
}

fn do_ocr<P: AsRef<Path>>(path: P) -> Result<DocDetails> {
    debug!("executing OCR on {}", path.as_ref().display());
    // NOTE: it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path.as_ref())?;
    Ok(DocDetails::new(path, lt.get_utf8_text()?))
}
