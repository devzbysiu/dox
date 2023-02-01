use std::fs::DirEntry;
use std::path::Path;

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}

pub trait PathRefExt {
    fn str(&self) -> &str;
    #[cfg(test)] // TODO: should this really be here?
    fn first_filename(&self) -> String;
}

impl<T: AsRef<Path>> PathRefExt for T {
    fn str(&self) -> &str {
        self.as_ref().to_str().expect("path is not utf8")
    }

    #[cfg(test)]
    fn first_filename(&self) -> String {
        self.as_ref()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .filename()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use std::fs::{read_dir, File};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_filename_in_dir_entry_ext() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        File::create(tmp_dir.path().join("test-file"))?;

        // when
        let entry = read_dir(&tmp_dir)?.next().unwrap()?;
        let filename = entry.file_name().to_str().unwrap().to_string();

        // then
        assert_eq!(filename, entry.filename());

        Ok(())
    }

    #[test]
    fn test_str_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let result = path.str();

        // then
        assert_eq!(path.to_str().unwrap(), result);
    }
}
