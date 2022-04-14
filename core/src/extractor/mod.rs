use crate::entities::document::DocDetails;
use crate::extension::Ext;
use crate::extractor::image::Ocr;
use crate::extractor::pdf::Pdf;

use std::path::PathBuf;

pub mod image;
pub mod pdf;

#[allow(clippy::module_name_repetitions)]
pub trait TextExtractor {
    fn extract_text(&self, path: &[PathBuf]) -> Vec<DocDetails>;
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

pub type Extractor = Box<dyn TextExtractor>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extractor_factory_with_correct_file() {
        // given
        let test_cases = vec![
            (Ext::Png, "res/doc1.png", "W dalszym ciągu uważamy"),
            (Ext::Jpg, "res/doc3.jpg", "Szanowny Panie"),
            (Ext::Webp, "res/doc4.webp", "Trybunału Konstytucyjnego"),
            (Ext::Pdf, "res/doc1.pdf", "Jak zainstalować scaner"),
        ];

        for test_case in test_cases {
            let ext = test_case.0;

            // when
            let extractor = ExtractorFactory::from_ext(&ext);
            let docs = extractor.extract_text(&[PathBuf::from(test_case.1)]);
            let doc = &docs[0];

            // then
            assert!(doc.body.contains(test_case.2));
        }
    }

    #[test]
    fn test_extractor_factory_with_wrong_file() {
        // given
        let ext = Ext::Pdf;

        // when
        let extractor = ExtractorFactory::from_ext(&ext);
        let docs = extractor.extract_text(&[PathBuf::from("res/doc1.png")]);

        // then
        assert!(docs.is_empty());
    }
}
