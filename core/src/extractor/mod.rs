use crate::extractor::image::Ocr;
use crate::extractor::pdf::Pdf;

use std::path::{Path, PathBuf};

pub mod image;
pub mod pdf;

// TODO: item name ends with mod name
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<FilenameToBody>;
}

pub struct FilenameToBody {
    pub filename: String,
    pub body: String,
}

impl FilenameToBody {
    fn new<P: AsRef<Path>, S: Into<String>>(path: P, body: S) -> Self {
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let body = body.into();
        Self { filename, body }
    }
}

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
