use crate::cfg::Config;
use crate::extractor::Ext;
use crate::helpers::PathBufExt;
use crate::result::Result;

use log::debug;
use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait FilePreprocessor {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()>;
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PreprocessorFactory;

impl PreprocessorFactory {
    pub fn from_ext(ext: &Ext, config: &Config) -> Box<dyn FilePreprocessor> {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(Image::new(config.thumbnails_dir.clone())),
            Ext::Pdf => Box::new(Pdf::new(config.thumbnails_dir.clone())),
        }
    }
}

pub struct Pdf {
    #[allow(unused)] // TODO: Remove that
    thumbnails_dir: PathBuf,
}

impl Pdf {
    fn new(thumbnails_dir: PathBuf) -> Self {
        Self { thumbnails_dir }
    }
}

impl FilePreprocessor for Pdf {
    fn preprocess(&self, _paths: &[PathBuf]) -> Result<()> {
        Ok(())
    }
}

pub struct Image {
    thumbnails_dir: PathBuf,
}

impl Image {
    fn new(thumbnails_dir: PathBuf) -> Self {
        Self { thumbnails_dir }
    }
}

impl FilePreprocessor for Image {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        for p in paths {
            // TODO: take care of this unwrap
            debug!("moving {} to {}", p.display(), self.thumbnails_dir.str());
            std::fs::copy(p, self.thumbnails_dir.join(p.file_name().unwrap())).unwrap();
        }
        Ok(())
    }
}
