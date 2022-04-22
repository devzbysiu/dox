//! Abstraction for everything related to thumbnail.
use crate::result::Result;

use std::fmt::Debug;
use std::path::Path;

/// Abstracts thumbnail generation. This trait is used to generate thumbnail for PDF documents so
/// they can be consumed the same way as images by client application.
#[allow(clippy::module_name_repetitions)]
pub trait ThumbnailGenerator: Debug {
    /// Generates thumbnail.
    fn generate(&self, pdf_path: &Path, out_path: &Path) -> Result<()>;
}
