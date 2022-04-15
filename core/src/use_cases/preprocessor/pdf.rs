use crate::entities::preprocessor::FilePreprocessor;
use crate::entities::thumbnail::ThumbnailGenerator;
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::thumbnail::ThumbnailGeneratorImpl;

use std::path::{Path, PathBuf};
use tracing::instrument;

#[derive(Debug)]
pub struct Pdf {
    thumbnails_dir: PathBuf,
    thumbnail_generator: ThumbnailGeneratorImpl,
}

impl Pdf {
    pub fn new<P: AsRef<Path>>(thumbnails_dir: P) -> Self {
        let thumbnails_dir = thumbnails_dir.as_ref().to_path_buf();
        let thumbnail_generator = ThumbnailGeneratorImpl;
        Self {
            thumbnails_dir,
            thumbnail_generator,
        }
    }

    fn thumbnail_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.thumbnails_dir.join(format!("{}.png", path.filestem()))
    }
}

impl FilePreprocessor for Pdf {
    #[instrument]
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        for pdf_path in paths {
            self.thumbnail_generator
                .generate(pdf_path, &self.thumbnail_path(pdf_path))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::helpers::DirEntryExt;

    use super::*;

    #[test]
    fn test_preprocess_with_correct_files() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Pdf::new(&tmp_dir);
        let paths = &[PathBuf::from("res/doc1.pdf")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(paths)?;
        let file = tmp_dir.path().read_dir()?.next().unwrap()?.filename();

        // then
        assert_eq!(file, "doc1.png");

        Ok(())
    }

    #[test]
    #[should_panic(expected = "PDF document is damaged")]
    fn test_preprocess_with_wrong_files() {
        // given
        let tmp_dir = tempdir().unwrap();
        let preprocessor = Pdf::new(tmp_dir);
        let paths = &[PathBuf::from("res/doc8.jpg")];

        // then
        preprocessor.preprocess(paths).unwrap(); // should panic
    }
}
