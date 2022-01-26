use anyhow::Result;
use leptess::LepTess;
use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

pub struct IndexTuple {
    pub filename: String,
    pub body: String,
}

impl IndexTuple {
    fn new<S: Into<String>, A: AsRef<Path>>(path: A, body: S) -> Self {
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let body = body.into();
        Self { filename, body }
    }
}

pub fn extract_text(paths: &[PathBuf]) -> Vec<IndexTuple> {
    debug!("extracting text...");
    paths
        .par_iter()
        .map(do_ocr)
        .filter_map(Result::ok)
        .collect::<Vec<IndexTuple>>()
}

fn do_ocr<P: AsRef<Path>>(path: P) -> Result<IndexTuple> {
    debug!("executing OCR on {}", path.as_ref().display());
    // NOTE: it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path.as_ref())?;
    Ok(IndexTuple::new(path, lt.get_utf8_text()?))
}
