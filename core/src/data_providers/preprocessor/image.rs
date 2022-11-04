use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::{GeneralErr, PreprocessorErr};
use crate::use_cases::services::preprocessor::FilePreprocessor;
use std::fs::{self, create_dir_all};

use std::path::Path;
use tracing::{debug, instrument};

/// Puts copy of an image to thumbnails directory.
///
/// It utilizes [`fs::copy`] function to move a copy to target directory. Thumbnails directory
/// comes from the configuration - see [`Config`](crate::configuration::cfg::Config).
#[derive(Debug)]
pub struct Image;

impl FilePreprocessor for Image {
    #[instrument(skip(self))]
    fn preprocess(&self, loc: &Location, target_dir: &Path) -> Result<Location, PreprocessorErr> {
        let Location::FS(paths) = loc;
        let mut result_paths = Vec::new();
        for p in paths {
            let ext = p.ext()?;
            if !ext.is_image() {
                return Err(PreprocessorErr::InvalidExtension(
                    GeneralErr::InvalidExtension,
                ));
            }
            let target_path = target_dir.join(p.rel_path());
            create_dir_all(target_path.parent_path())?;
            debug!("moving '{:?}' to '{}'", p, target_path.display());
            fs::copy(p, &target_path)?;
            result_paths.push(target_path.into());
        }
        Ok(Location::FS(result_paths))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::entities::location::SafePathBuf;
    use crate::helpers::DirEntryExt;

    use anyhow::Result;
    use claim::assert_err;
    use tempfile::tempdir;

    #[test]
    fn image_preprocessor_puts_image_files_in_user_dir() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Image;
        let paths = vec![SafePathBuf::from("res/doc1.png")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(&Location::FS(paths), tmp_dir.path())?;
        let user_dir = tmp_dir.path().read_dir()?.next().unwrap()?;

        // then
        assert_eq!(user_dir.filename(), "res");
        assert_eq!(user_dir.path().first_filename(), "doc1.png");

        Ok(())
    }

    #[test]
    fn image_preprocessor_fails_with_non_image_files() -> Result<()> {
        // given
        let tmp_dir = tempdir().unwrap();
        let preprocessor = Image;
        let paths = vec![SafePathBuf::from("res/doc1.pdf")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // TODO: currently, this just copies the file to the thumbnails_dir without
        // checking if this is the correct file type. Potentially this should be checked
        // and error should be thrown (and this should be consistent with Pdf preprocessor)
        // when
        let res = preprocessor.preprocess(&Location::FS(paths), tmp_dir.path());

        // then
        assert_err!(res);

        Ok(())
    }
}
