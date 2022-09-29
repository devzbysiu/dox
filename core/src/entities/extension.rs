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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn supported_extension_can_be_created_from_string() {
        // given
        let test_cases = [
            ("png", Ext::Png),
            ("jpg", Ext::Jpg),
            ("jpeg", Ext::Jpg),
            ("webp", Ext::Webp),
            ("pdf", Ext::Pdf),
        ];

        for test_case in test_cases {
            // then
            assert_eq!(Ext::from(test_case.0), test_case.1);
        }
    }
}
