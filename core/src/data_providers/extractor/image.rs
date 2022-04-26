//! Allows to extract text from image using OCR.
use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::extractor::TextExtractor;

use leptess::LepTess;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

/// Extracts text from the image.
///
/// It's using [`LepTess`] to extract text from the image. All images pointed by `paths` are
/// processed in parallel thanks to [`ParallelIterator`].
#[derive(Debug, Default)]
pub struct FromImage;

impl TextExtractor for FromImage {
    #[instrument]
    fn extract_text(&self, paths: &[PathBuf]) -> Vec<DocDetails> {
        paths
            .par_iter()
            .map(ocr)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>()
    }

    #[allow(unused)]
    #[instrument]
    fn extract_text_from_location(&self, location: &Location) -> Result<Vec<DocDetails>> {
        let Location::FileSystem(paths) = location;
        Ok(paths
            .par_iter()
            .map(ocr)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>())
    }
}

fn ocr<P: AsRef<Path>>(path: P) -> Result<DocDetails> {
    debug!("executing OCR on {}", path.as_ref().display());
    // NOTE: it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path.as_ref())?;
    Ok(DocDetails::new(&path, lt.get_utf8_text()?, path.filename()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_text() {
        // given
        let ocr = FromImage;
        let paths = vec![PathBuf::from("res/doc1.png"), PathBuf::from("res/doc3.jpg")];

        // when
        let mut result = ocr.extract_text(&paths);
        result.sort();

        // then
        let first_doc_details = &result[0];
        let second_doc_details = &result[1];

        assert!(first_doc_details.body.contains("W odpowiedzi na pismo"));
        assert_eq!(first_doc_details.filename, "doc1.png");
        assert_eq!(first_doc_details.thumbnail, "doc1.png");

        assert!(second_doc_details.body.contains("Szanowny Panie"));
        assert_eq!(second_doc_details.filename, "doc3.jpg");
        assert_eq!(second_doc_details.thumbnail, "doc3.jpg");
    }

    #[test]
    fn test_extract_text_with_non_existing_paths() {
        // given
        let ocr = FromImage;
        let paths = vec![
            PathBuf::from("not/existing-1"),
            PathBuf::from("not/existing-2"),
        ];

        // when
        let result = ocr.extract_text(&paths);

        // then
        assert!(result.is_empty());
    }
}
