use crate::result::Result;

use std::{fmt::Debug, path::PathBuf};

pub trait ThumbnailGenerator: Debug {
    fn generate(&self, pdf_path: &PathBuf, out_path: &PathBuf) -> Result<()>;
}
