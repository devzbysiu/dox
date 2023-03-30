//! Allows to extract text from PDF.
use crate::entities::document::DocDetails;
use crate::entities::file::{Filename, Thumbnailname};
use crate::entities::location::{Location, SafePathBuf};
use crate::entities::user::User;
use crate::result::ExtractorErr;
use crate::use_cases::services::extractor::DataExtractor;

use pdf_extract::extract_text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::convert::TryFrom;
use std::fmt::Debug;
use tracing::{instrument, trace};

/// Extracts text from PDF file.
///
/// It uses [`extract_text`] to extract text. All files pointed by `paths` are processed in
/// parallel.
#[derive(Debug, Default)]
pub struct FromPdf;

impl DataExtractor for FromPdf {
    #[instrument(skip(self))]
    fn extract_data(&self, location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        let Location::FS(paths) = location;
        Ok(paths
            .par_iter()
            .map(extract)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>())
    }
}

#[instrument]
fn extract(path: &SafePathBuf) -> Result<DocDetails, ExtractorErr> {
    let filename = Filename::from(path);
    let text = extract_text(path)?;
    let thumbnailname = Thumbnailname::new(thumbnail_name(path))?;
    let user = User::try_from(path)?;
    trace!("extracted text: '{}'", text);
    Ok(DocDetails::new(filename, text, thumbnailname, user))
}

fn thumbnail_name(path: &SafePathBuf) -> String {
    format!("{}.png", path.filestem())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::entities::location::SafePathBuf;
    use anyhow::Result;

    #[test]
    fn test_extract_text() -> Result<()> {
        // given
        let pdf = FromPdf;
        let paths = vec![
            SafePathBuf::from("res/doc1.pdf"),
            SafePathBuf::from("res/doc2.pdf"),
        ];

        // when
        let mut result = pdf.extract_data(&Location::FS(paths))?;
        result.sort();

        // then
        let first_doc = &result[0];
        let second_doc = &result[1];

        assert!(first_doc.body.contains("Jak zainstalować scaner"));
        assert_eq!(first_doc.filename, Filename::new("doc1.pdf")?);
        assert_eq!(first_doc.thumbnail, Thumbnailname::new("doc1.png")?);

        assert!(second_doc.body.contains("Podmiot powierzający"));
        assert_eq!(second_doc.filename, Filename::new("doc2.pdf")?);
        assert_eq!(second_doc.thumbnail, Thumbnailname::new("doc2.png")?);

        Ok(())
    }
}
