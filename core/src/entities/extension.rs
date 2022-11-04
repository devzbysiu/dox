//! Represents extension of the documents appearing in the system.
//!
//! Indexing and preprocessing strategies are based on the extension. See
//! [`ExtractorFactoryImpl`](crate::data_providers::extractor::ExtractorFactoryImpl)
//! and [`PreprocessorFactoryImpl`](crate::data_providers::preprocessor::PreprocessorFactoryImpl).

use std::convert::TryFrom;

use crate::result::GeneralErr;

/// File extension.
///
/// Contains all currently supported filetypes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ext {
    Png,
    Jpg,
    Webp,
    Pdf,
}

impl Ext {
    pub fn is_image(&self) -> bool {
        match self {
            Ext::Png | Ext::Jpg | Ext::Webp => true,
            Ext::Pdf => false,
        }
    }
}

impl TryFrom<String> for Ext {
    type Error = GeneralErr;

    fn try_from(ext: String) -> Result<Self, Self::Error> {
        let ext = ext.as_ref();
        Ok(match ext {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpg,
            "webp" => Self::Webp,
            "pdf" => Self::Pdf,
            _ => return Err(GeneralErr::InvalidExtension),
        })
    }
}

impl From<&str> for Ext {
    fn from(ext: &str) -> Self {
        match ext {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpg,
            "webp" => Self::Webp,
            "pdf" => Self::Pdf,
            _ => panic!("failed to create extension from '{}'", ext),
        }
    }
}
