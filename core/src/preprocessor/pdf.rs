use crate::preprocessor::FilePreprocessor;
use crate::result::Result;

use std::path::PathBuf;

pub struct Pdf {
    #[allow(unused)] // TODO: Remove that
    thumbnails_dir: PathBuf,
}

impl Pdf {
    pub fn new(thumbnails_dir: PathBuf) -> Self {
        Self { thumbnails_dir }
    }
}

impl FilePreprocessor for Pdf {
    fn preprocess(&self, _paths: &[PathBuf]) -> Result<()> {
        Ok(())
    }
}
