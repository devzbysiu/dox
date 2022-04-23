use crate::data_providers::extractor::image::FromImage;
use crate::data_providers::extractor::pdf::FromPdf;
use crate::entities::extension::Ext;
use crate::use_cases::extractor::{Extractor, ExtractorFactory};

pub mod image;
pub mod pdf;

/// Creates specific extractor based on the extension.
///
/// Each filetype requires different way of extracting text. For example extracting text from PDF
/// file differs from extracting text from regular image (see
/// [`FromPdf`](crate::data_providers::extractor::pdf::FromPdf) and
/// [`FromImage`](crate::data_providers::extractor::image::FromImage)).
///
/// The type of file is decided based on the file extension.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ExtractorFactoryImpl;

impl ExtractorFactory for ExtractorFactoryImpl {
    fn from_ext(&self, ext: &Ext) -> Extractor {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(FromImage),
            Ext::Pdf => Box::new(FromPdf),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extractor_factory_with_correct_file() {
        // given
        let test_cases = vec![
            (Ext::Png, "res/doc1.png", "W dalszym ciągu uważamy"),
            (Ext::Jpg, "res/doc3.jpg", "Szanowny Panie"),
            (Ext::Webp, "res/doc4.webp", "Trybunału Konstytucyjnego"),
            (Ext::Pdf, "res/doc1.pdf", "Jak zainstalować scaner"),
        ];
        let extractor_factory = ExtractorFactoryImpl;

        for test_case in test_cases {
            let ext = test_case.0;

            // when
            let extractor = extractor_factory.from_ext(&ext);
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
        let extractor_factory = ExtractorFactoryImpl;

        // when
        let extractor = extractor_factory.from_ext(&ext);
        let docs = extractor.extract_text(&[PathBuf::from("res/doc1.png")]);

        // then
        assert!(docs.is_empty());
    }
}
