//! This is a specific implementation of a [`crate::use_cases::persistence`] mod.
//!
//! It uses just regular File System primitives to provide persistence.
use crate::entities::location::SafePathBuf;
use crate::helpers::PathRefExt;
use crate::result::FsErr;
use crate::use_cases::fs::Filesystem;

use std::fs::{self, create_dir_all};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, instrument};

pub struct LocalFs;

impl Filesystem for LocalFs {
    #[instrument(skip(self, buf))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        let parent_dir = uri.parent_path();
        if !parent_dir.exists() {
            create_dir_all(parent_dir)?;
            // NOTE: this is needed because when file creation happens immediately after directory
            // creation, then the file creation event is not yet registered by filesystem watching
            // library
            thread::sleep(Duration::from_secs(1)); // allow to start watching for new directory
        }
        fs::write(uri, buf)?;
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        debug!("returning file under '{}'", uri.display());
        Ok(fs::read(uri)?)
    }

    fn rm_file(&self, _path: &SafePathBuf) -> Result<(), FsErr> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use claim::{assert_gt, assert_lt, assert_matches, assert_ok_eq};
    use fake::faker::filesystem::en::FilePath;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use fs::read_to_string;
    use std::io::ErrorKind;
    use std::time::Instant;
    use tempfile::tempdir;

    #[test]
    fn save_correctly_saves_the_data_to_path() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("file");
        let fs = LocalFs;

        // when
        fs.save(file_path.clone(), data.as_ref())?;

        // then
        assert_eq!(read_to_string(file_path)?, data);

        Ok(())
    }

    #[test]
    fn parent_dir_is_first_created_if_not_exists() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("not-existing-parent-dir/file");
        let fs = LocalFs;

        // when
        fs.save(file_path.clone(), data.as_ref())?;

        // then
        assert_eq!(read_to_string(file_path)?, data);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "failed to get parent dir")]
    fn it_panics_when_could_not_get_parent_dir() {
        // given
        let data: String = Paragraph(1..2).fake();
        let fs = LocalFs;

        // then
        fs.save(PathBuf::from("/"), data.as_ref()).unwrap(); // should panic
    }

    #[test]
    fn it_takes_at_least_one_second_to_save_file_when_parent_dir_not_exists() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("not-existing-parent-dir/file");
        let fs = LocalFs;
        let now = Instant::now();

        // when
        fs.save(file_path, data.as_ref())?;

        // then
        let elapsed = now.elapsed();
        assert_gt!(elapsed, Duration::from_secs(1));

        Ok(())
    }

    #[test]
    fn it_takes_less_than_second_to_save_a_file_when_parent_dir_exists() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("file");
        let fs = LocalFs;
        let now = Instant::now();

        // when
        fs.save(file_path, data.as_ref())?;

        // then
        let elapsed = now.elapsed();
        assert_lt!(elapsed, Duration::from_secs(1));

        Ok(())
    }

    #[test]
    fn load_correctly_loads_data_from_path() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("file");
        fs::write(&file_path, &data)?;
        let fs = LocalFs;

        // when
        let res = fs.load(file_path);

        // then
        assert_ok_eq!(res, data.as_bytes().to_vec());

        Ok(())
    }

    #[test]
    fn it_returns_io_error_when_source_path_does_not_exist() {
        // given
        let file_path = FilePath().fake();
        let fs = LocalFs;

        // when
        let res = fs.load(file_path);

        // then
        assert_matches!(res, Err(FsErr::IoError(e)) if e.kind() == ErrorKind::NotFound);
    }
}
