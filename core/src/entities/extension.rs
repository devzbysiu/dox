//! Represents extension of the documents appearing in the system.
//!
//! Indexing and preprocessing strategies are based on the extension. See
//! [`ExtractorFactoryImpl`](crate::data_providers::extractor::ExtractorFactoryImpl)
//! and [`ThumbnailerFactoryImpl`](crate::data_providers::thumbnailer::ThumbnailerFactoryImpl).

use crate::result::GeneralErr;

use enum_iterator::{all, Sequence};
use std::convert::TryFrom;

/// File extension.
///
/// Contains all currently supported filetypes.
#[derive(Debug, Clone, PartialEq, Eq, Sequence)]
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

pub fn supported_extensions() -> Vec<Ext> {
    all::<Ext>().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn supported_extensions_returns_all_extensions_from_enum() {
        // given
        let all_extensions = vec![Ext::Png, Ext::Jpg, Ext::Webp, Ext::Pdf];

        // when
        let supported_extensions = supported_extensions();

        // then
        assert_eq!(all_extensions, supported_extensions);
    }
}
