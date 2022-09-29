//! Represents extension of the documents appearing in the system.
//!
//! Indexing and preprocessing strategies are based on the extension. See
//! [`ExtractorFactoryImpl`](crate::data_providers::extractor::ExtractorFactoryImpl)
//! and [`PreprocessorFactoryImpl`](crate::data_providers::preprocessor::PreprocessorFactoryImpl).

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

impl<S: Into<String>> From<S> for Ext {
    fn from(ext: S) -> Self {
        let ext = ext.into();
        match ext.as_ref() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpg,
            "webp" => Self::Webp,
            "pdf" => Self::Pdf,
            _ => panic!("failed to create extension from '{}'", ext),
        }
    }
}
