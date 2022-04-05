use crate::extractor::image::Ocr;
use crate::extractor::pdf::Pdf;
use crate::helpers::PathRefExt;

use std::path::{Path, PathBuf};

pub mod image;
pub mod pdf;

#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<DocDetails>;
}

#[derive(Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct DocDetails {
    pub filename: String,
    pub body: String,
    pub thumbnail: String,
}

impl DocDetails {
    pub fn new<P: AsRef<Path>, S: Into<String>>(path: P, body: S, thumbnail: S) -> Self {
        Self {
            filename: path.filename(),
            body: body.into(),
            thumbnail: thumbnail.into(),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ExtractorFactory;

impl ExtractorFactory {
    pub fn from_ext(ext: &Ext) -> Extractor {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(Ocr),
            Ext::Pdf => Box::new(Pdf),
        }
    }
}

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

pub type Extractor = Box<dyn TextExtractor>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extractor_factory() {
        // given
        let ext = Ext::Pdf;

        // when
        let extractor = ExtractorFactory::from_ext(&ext);
        let docs = extractor.extract_text(&[PathBuf::from("res/doc1.pdf")]);
        let doc = &docs[0];

        // then
        assert!(doc.body.contains("Jak zainstalować scaner"));
    }
}
