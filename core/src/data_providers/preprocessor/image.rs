use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::Result;
use crate::use_cases::preprocessor::FilePreprocessor;
use std::fs;

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
    fn preprocess(&self, location: &Location, thumbnails_dir: &Path) -> Result<()> {
        let Location::FileSystem(paths) = location;
        for p in paths {
            debug!("moving {} to {}", p.display(), thumbnails_dir.str());
            fs::copy(
                p,
                thumbnails_dir.join(format!(
                    "{}/{}",
                    p.parent().expect("failed to get parent dir").filename(), // TODO: maybe this should be moved to helpers?
                    p.filename()
                )),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_preprocess_with_correct_files() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Image;
        let paths = vec![PathBuf::from("res/doc1.png")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // when
        preprocessor.preprocess(&Location::FileSystem(paths), tmp_dir.path())?;
        let file = tmp_dir.path().first_filename()?;

        // then
        assert_eq!(file, "doc1.png");

        Ok(())
    }

    #[test]
    fn test_preprocess_with_wrong_files() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let preprocessor = Image;
        let paths = vec![PathBuf::from("res/doc1.pdf")];
        let is_empty = tmp_dir.path().read_dir()?.next().is_none();
        assert!(is_empty);

        // TODO: currently, this just copies the file to the thumbnails_dir without
        // checking if this is the correct file type. Potentially this should be checked
        // and error should be thrown (and this should be consistent with Pdf preprocessor)
        // when
        preprocessor.preprocess(&Location::FileSystem(paths), tmp_dir.path())?;
        let file = tmp_dir.path().first_filename()?;

        // then
        assert_eq!(file, "doc1.pdf");

        Ok(())
    }
}
