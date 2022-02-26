use log::debug;

use crate::helpers::PathRefExt;
use crate::preprocessor::FilePreprocessor;
use crate::result::Result;
use crate::thumbnail;

use std::path::{Path, PathBuf};

pub struct Pdf {
    thumbnails_dir: PathBuf,
}

impl Pdf {
    pub fn new(thumbnails_dir: PathBuf) -> Self {
        Self { thumbnails_dir }
    }

    fn thumbnail_path<P: AsRef<Path>>(&self, pdf_path: P) -> PathBuf {
        self.thumbnails_dir.join(pdf_path.filename())
    }
}

impl FilePreprocessor for Pdf {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        debug!("generating thumbnails for paths: {:?}", paths);
        for pdf_path in paths {
            thumbnail::generate(pdf_path, &self.thumbnail_path(pdf_path))?;
        }
        Ok(())
    }
}
