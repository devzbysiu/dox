use crate::cfg::Config;
use crate::extractor::Ext;
use crate::preprocessor::image::Image;
use crate::preprocessor::pdf::Pdf;
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
    pub fn from_ext(ext: &Ext, config: &Config) -> Preprocessor {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(Image::new(config.thumbnails_dir.clone())),
            Ext::Pdf => Box::new(Pdf::new(config.thumbnails_dir.clone())),
        }
    }
}

pub type Preprocessor = Box<dyn FilePreprocessor>;
