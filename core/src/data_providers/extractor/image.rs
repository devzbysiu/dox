//! Allows to extract text from image using OCR.
use crate::entities::document::DocDetails;
use crate::entities::file::{Filename, Thumbnailname};
use crate::entities::location::{Location, SafePathBuf};
use crate::entities::user::User;
use crate::result::ExtractorErr;
use crate::use_cases::services::extractor::DataExtractor;

use leptess::LepTess;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::convert::TryFrom;
use tracing::{debug, instrument};

/// Extracts text from the image.
///
/// It's using [`LepTess`] to extract text from the image. All images pointed by `paths` are
/// processed in parallel thanks to [`ParallelIterator`].
#[derive(Debug, Default)]
pub struct FromImage;

impl DataExtractor for FromImage {
    #[instrument(skip(self))]
    fn extract_data(&self, location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        let Location::FS(paths) = location;
        Ok(paths
            .par_iter()
            .map(extract_details)
            .filter_map(Result::ok)
            .collect::<Vec<DocDetails>>())
    }
}

fn extract_details(path: &SafePathBuf) -> Result<DocDetails, ExtractorErr> {
    debug!("executing OCR on {:?}", path);
    // NOTE: it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path)?;
    let filename = Filename::from(path);
    let body = lt.get_utf8_text()?;
    let thumbnailname = Thumbnailname::from(path);
    let user = User::try_from(path)?;
    Ok(DocDetails::new(filename, body, thumbnailname, user))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::entities::location::SafePathBuf;
    use anyhow::Result;

    #[test]
    fn test_extract_text() -> Result<()> {
        // given
        let ocr = FromImage;
        let paths = vec![
            SafePathBuf::from("res/doc1.png"),
            SafePathBuf::from("res/doc3.jpg"),
        ];

        // when
        let mut result = ocr.extract_data(&Location::FS(paths))?;
        result.sort();

        // then
        let first_doc = &result[0];
        let second_doc = &result[1];

        assert!(first_doc.body.contains("W odpowiedzi na pismo"));
        assert_eq!(first_doc.filename, Filename::new("doc1.png")?);
        assert_eq!(first_doc.thumbnail, Thumbnailname::new("doc1.png")?);

        assert!(second_doc.body.contains("Szanowny Panie"));
        assert_eq!(second_doc.filename, Filename::new("doc3.jpg")?);
        assert_eq!(second_doc.thumbnail, Thumbnailname::new("doc3.jpg")?);

        Ok(())
    }
}
