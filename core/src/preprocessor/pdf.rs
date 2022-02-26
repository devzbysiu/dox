use log::debug;

use crate::preprocessor::FilePreprocessor;
use crate::result::Result;
use crate::thumbnail;

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
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        debug!("generating thumbnails for paths: {:?}", paths);
        for pdf_path in paths {
            thumbnail::generate(
                pdf_path,
                // TODO: take care of this unwrap
                &self.thumbnails_dir.join(pdf_path.file_name().unwrap()),
            )?;
        }
        Ok(())
    }
}
