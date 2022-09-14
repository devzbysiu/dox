use crate::data_providers::preprocessor::image::Image;
use crate::data_providers::preprocessor::pdf::Pdf;
use crate::entities::extension::Ext;
use crate::use_cases::services::preprocessor::{Preprocessor, PreprocessorFactory};

pub mod image;
pub mod pdf;

/// Creates specific [`Preprocessor`] based on the extension.
///
/// Each filetype requires different way of preprocessing a file. For example preprocessing a PDF
/// file differs from preprocessing regular image (see
/// [`FromPdf`](crate::data_providers::preprocessor::pdf::Pdf) and
/// [`FromImage`](crate::data_providers::preprocessor::image::Image)).
///
/// The type of a file is decided based on the file extension.
#[derive(Debug)]
pub struct PreprocessorFactoryImpl;

impl PreprocessorFactory for PreprocessorFactoryImpl {
    fn make(&self, ext: &Ext) -> Preprocessor {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(Image),
            Ext::Pdf => Box::new(Pdf),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::entities::location::Location;
    use crate::helpers::PathRefExt;
    use crate::result::Result;

    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_processor_factory_with_correct_file() -> Result<()> {
        // given
        let test_cases = vec![
            (Ext::Png, "res/doc1.png", "doc1.png"),
            (Ext::Jpg, "res/doc3.jpg", "doc3.jpg"),
            (Ext::Webp, "res/doc4.webp", "doc4.webp"),
            (Ext::Pdf, "res/doc1.pdf", "doc1.png"),
        ];
        let preprocessor_factory = PreprocessorFactoryImpl;

        for test_case in test_cases {
            let ext = test_case.0;

            let tmp_dir = tempdir()?;
            let paths = vec![PathBuf::from(test_case.1)];

            // when
            let extractor = preprocessor_factory.make(&ext);
            extractor.preprocess(&Location::FileSystem(paths), tmp_dir.path())?;

            // then
            assert_eq!(tmp_dir.path().first_filename()?, "res");
            assert_eq!(tmp_dir.path().join("res").first_filename()?, test_case.2);
        }

        Ok(())
    }
}
