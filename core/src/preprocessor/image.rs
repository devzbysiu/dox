use crate::helpers::{PathBufExt, PathRefExt};
use crate::preprocessor::FilePreprocessor;
use crate::result::Result;

use log::debug;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Image {
    thumbnails_dir: PathBuf,
}

impl Image {
    pub fn new<P: AsRef<Path>>(thumbnails_dir: P) -> Self {
        let thumbnails_dir = thumbnails_dir.as_ref().to_path_buf();
        Self { thumbnails_dir }
    }
}

impl FilePreprocessor for Image {
    fn preprocess(&self, paths: &[PathBuf]) -> Result<()> {
        for p in paths {
            debug!("moving {} to {}", p.display(), self.thumbnails_dir.str());
            std::fs::copy(p, self.thumbnails_dir.join(p.filename()))?;
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
        let preprocessor = Image::new(&tmp_dir);
        let paths = &[PathBuf::from("res/doc1.png")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(paths)?;
        let file = tmp_dir.path().read_dir()?.next().unwrap()?.filename();

        // then
        assert_eq!(file, "doc1.png");

        Ok(())
    }
}
