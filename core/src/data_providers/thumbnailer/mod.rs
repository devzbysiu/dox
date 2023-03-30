use tracing::instrument;

use crate::data_providers::thumbnailer::image::ImageThumbnailer;
use crate::data_providers::thumbnailer::pdf::PdfThumbnailer;
use crate::entities::extension::Ext;
use crate::use_cases::services::thumbnailer::{Thumbnailer, ThumbnailerFactory};

#[cfg(test)]
pub use test::DirEntryExt;

pub mod image;
pub mod pdf;

/// Creates specific [`Thumbnailer`] based on the extension.
///
/// Each filetype requires different way of creating a thumbnail. For example creating a thumbnail
/// of PDF file differs from creating a thumbnail of regular image (see
/// [`FromPdf`](crate::data_providers::thumbnailer::pdf::Pdf) and
/// [`FromImage`](crate::data_providers::thumbnailer::image::Image)).
///
/// The type of a file is decided based on the file extension.
#[derive(Debug)]
pub struct ThumbnailerFactoryImpl;

impl ThumbnailerFactory for ThumbnailerFactoryImpl {
    #[instrument(skip(self))]
    fn make(&self, ext: &Ext) -> Thumbnailer {
        match ext {
            Ext::Png | Ext::Jpg | Ext::Webp => Box::new(ImageThumbnailer),
            Ext::Pdf => Box::new(PdfThumbnailer),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::entities::location::{Location, SafePathBuf};
    use crate::helpers::PathRefExt;

    use anyhow::Result;
    use std::fs::{read_dir, DirEntry, File};
    use tempfile::tempdir;

    #[test]
    fn test_thumbnailer_factory_with_correct_file() -> Result<()> {
        // given
        let test_cases = vec![
            (Ext::Png, "res/doc1.png", "doc1.png"),
            (Ext::Jpg, "res/doc3.jpg", "doc3.jpg"),
            (Ext::Webp, "res/doc4.webp", "doc4.webp"),
            (Ext::Pdf, "res/doc1.pdf", "doc1.png"),
        ];
        let thumbnailer_factory = ThumbnailerFactoryImpl;

        for test_case in test_cases {
            let ext = test_case.0;

            let tmp_dir = tempdir()?;
            let paths = vec![SafePathBuf::from(test_case.1)];

            // when
            let extractor = thumbnailer_factory.make(&ext);
            extractor.mk_thumbnail(&Location::FS(paths), tmp_dir.path())?;

            // then
            assert_eq!(tmp_dir.path().first_filename(), "res");
            assert_eq!(tmp_dir.path().join("res").first_filename(), test_case.2);
        }

        Ok(())
    }

    pub trait DirEntryExt {
        fn name(&self) -> String;
    }

    impl DirEntryExt for DirEntry {
        fn name(&self) -> String {
            self.file_name().to_str().unwrap().to_string()
        }
    }

    #[test]
    fn test_name_in_dir_entry_ext() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        File::create(tmp_dir.path().join("test-file"))?;

        // when
        let entry = read_dir(&tmp_dir)?.next().unwrap()?;
        let filename = entry.file_name().to_str().unwrap().to_string();

        // then
        assert_eq!(filename, entry.name());

        Ok(())
    }
}
