use crate::cfg::Config;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::Result;

use std::path::PathBuf;

#[allow(clippy::module_name_repetitions)]
pub trait FilePreprocessor {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()>;
    fn preprocess_location(&self, location: &Location) -> Result<()>;
}

pub trait PreprocessorFactory: Sync + Send {
    fn from_ext(&self, ext: &Ext, config: &Config) -> Preprocessor;
}

pub type Preprocessor = Box<dyn FilePreprocessor>;
