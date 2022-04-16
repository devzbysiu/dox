use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::extractor::TextExtractor;

use pdf_extract::extract_text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use tracing::instrument;

#[derive(Debug, Default)]
pub struct Pdf;

impl TextExtractor for Pdf {
    #[instrument]
    fn extract_text(&self, paths: &[PathBuf]) -> Vec<DocDetails> {
        paths
            .par_iter()
            .map(extract)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>()
    }

    #[allow(unused)]
    #[instrument]
    fn extract_text_from_location(&self, location: &Location) -> Result<Vec<DocDetails>> {
        unimplemented!()
    }
}

#[instrument]
fn extract<P: AsRef<Path> + Debug>(path: P) -> Result<DocDetails> {
    let path = path.as_ref();
    Ok(DocDetails::new(path, extract_text(path)?, thumbnail(path)))
}

fn thumbnail<P: AsRef<Path>>(path: P) -> String {
    format!("{}.png", path.filestem())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_text() {
        // given
        let pdf = Pdf;
        let paths = vec![PathBuf::from("res/doc1.pdf"), PathBuf::from("res/doc2.pdf")];

        // when
        let mut result = pdf.extract_text(&paths);
        result.sort();

        // then
        let first_doc_details = &result[0];
        let second_doc_details = &result[1];

        assert!(first_doc_details.body.contains("Jak zainstalować scaner"));
        assert_eq!(first_doc_details.filename, "doc1.pdf");
        assert_eq!(first_doc_details.thumbnail, "doc1.png");

        assert!(second_doc_details.body.contains("Podmiot powierzający"));
        assert_eq!(second_doc_details.filename, "doc2.pdf");
        assert_eq!(second_doc_details.thumbnail, "doc2.png");
    }

    #[test]
    fn test_extract_text_with_non_existing_paths() {
        // given
        let pdf = Pdf;
        let paths = vec![
            PathBuf::from("not/existing-1"),
            PathBuf::from("not/existing-2"),
        ];

        // when
        let result = pdf.extract_text(&paths);

        // then
        assert!(result.is_empty());
    }
}
