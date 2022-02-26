use crate::cfg::Config;
use crate::extractor::Ext;
use crate::preprocessor::image::Image;
use crate::result::Result;

use std::path::PathBuf;

mod image;
mod pdf;

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
