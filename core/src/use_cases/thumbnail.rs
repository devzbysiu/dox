use crate::result::Result;

use std::fmt::Debug;
use std::path::PathBuf;

pub trait ThumbnailGenerator: Debug {
    fn generate(&self, pdf_path: &PathBuf, out_path: &PathBuf) -> Result<()>;
}
