use crate::helpers::{PathBufExt, PathRefExt};
use crate::preprocessor::FilePreprocessor;
use crate::result::Result;

use log::debug;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Image {
    thumbnails_dir: PathBuf,
}

impl Image {
    pub fn new(thumbnails_dir: PathBuf) -> Self {
        Self { thumbnails_dir }
    }
}

impl FilePreprocessor for Image {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        for p in paths {
            debug!("moving {} to {}", p.display(), self.thumbnails_dir.str());
            std::fs::copy(p, self.thumbnails_dir.join(p.filename()))?;
        }
        Ok(())
    }
}
