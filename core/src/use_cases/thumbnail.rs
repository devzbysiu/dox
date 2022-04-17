use crate::result::Result;

use std::fmt::Debug;
use std::path::Path;

#[allow(clippy::module_name_repetitions)]
pub trait ThumbnailGenerator: Debug {
    fn generate(&self, pdf_path: &Path, out_path: &Path) -> Result<()>;
}
