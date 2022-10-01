//! This is a specific implementation of a [`crate::use_cases::persistence`] mod.
//!
//! It uses just regular File System primitives to provide persistence.
use crate::helpers::PathRefExt;
use crate::result::PersistenceErr;
use crate::use_cases::persistence::DataPersistence;

use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, instrument};

pub struct FsPersistence;

impl DataPersistence for FsPersistence {
    #[instrument(skip(self, buf))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), PersistenceErr> {
        let parent_dir = uri.parent_path();
        if !parent_dir.exists() {
            create_dir_all(parent_dir)?;
            // NOTE: this is needed because when file creation happens immediately after directory
            // creation, then the file creation event is not yet registered by filesystem watching
            // library
            thread::sleep(Duration::from_secs(1)); // allow to start watching for new directory
        }
        let mut file = File::create(uri)?;
        file.write_all(buf)?;
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Option<Vec<u8>>, PersistenceErr> {
        debug!("returning file under '{}'", uri.display());
        Ok(Some(fs::read(uri)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use claim::{assert_gt, assert_lt, assert_ok_eq};
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
        let persistence = FsPersistence;

        // when
        persistence.save(file_path.clone(), data.as_ref())?;

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
        let persistence = FsPersistence;

        // when
        persistence.save(file_path.clone(), data.as_ref())?;

        // then
        assert_eq!(read_to_string(file_path)?, data);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "failed to get parent dir")]
    fn it_panics_when_could_not_get_parent_dir() {
        // given
        let data: String = Paragraph(1..2).fake();
        let persistence = FsPersistence;

        // then
        persistence.save(PathBuf::from("/"), data.as_ref()).unwrap(); // should panic
    }

    #[test]
    fn it_takes_at_least_one_second_to_save_file_when_parent_dir_not_exists() -> Result<()> {
        // given
        let data: String = Paragraph(1..2).fake();
        let target_dir = tempdir()?;
        let file_path = target_dir.path().join("not-existing-parent-dir/file");
        let persistence = FsPersistence;
        let now = Instant::now();

        // when
        persistence.save(file_path, data.as_ref())?;

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
        let persistence = FsPersistence;
        let now = Instant::now();

        // when
        persistence.save(file_path, data.as_ref())?;

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
        let persistence = FsPersistence;

        // when
        let res = persistence.load(file_path);

        // then
        assert_ok_eq!(res, Some(data.as_bytes().to_vec()));

        Ok(())
    }

    #[test]
    fn it_returns_io_error_when_source_path_does_not_exist() {
        // given
        let file_path = FilePath().fake();
        let persistence = FsPersistence;

        // when
        let res = persistence.load(file_path);

        // then
        if let Err(PersistenceErr::IoError(e)) = res {
            assert!(e.kind() == ErrorKind::NotFound);
        } else {
            panic!("Invalid result returned");
        }
    }
}
