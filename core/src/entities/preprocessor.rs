use super::extension::Ext;

use crate::cfg::Config;
use crate::result::Result;

use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait FilePreprocessor {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()>;
}

pub trait PreprocessorFactory {
    fn from_ext(ext: &Ext, config: &Config) -> Preprocessor;
}

pub type Preprocessor = Box<dyn FilePreprocessor>;
