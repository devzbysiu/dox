use crate::extractor::Ext;

use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}

pub trait PathExt {
    fn ext(&self) -> Ext;
    fn string(&self) -> String;
}

impl PathExt for Path {
    fn ext(&self) -> Ext {
        Ext::from(self.extension().unwrap().to_str().unwrap())
    }

    fn string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

pub trait PathBufExt {
    fn str(&self) -> &str;
}

impl PathBufExt for PathBuf {
    fn str(&self) -> &str {
        self.to_str().expect("path is not utf8")
    }
}

pub trait PathRefExt {
    fn filestem(&self) -> String;
    fn filename(&self) -> String;
}

impl<T: AsRef<Path>> PathRefExt for T {
    fn filestem(&self) -> String {
        let path = self.as_ref();
        path.file_stem()
            .unwrap_or_else(|| panic!("path '{}' does not have a filestem", path.display()))
            .to_string_lossy()
            .to_string()
    }

    fn filename(&self) -> String {
        let path = self.as_ref();
        path.file_name()
            .unwrap_or_else(|| panic!("path '{}' does not have a filename", path.display()))
            .to_string_lossy()
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::{
        fs::{read_dir, File},
        io::Write,
    };
    use tempfile::tempdir;

    #[test]
    fn test_filename_in_dir_entry_ext() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        let mut f = File::create(tmp_dir.path().join("test-file"))?;
        f.write_all(b"test")?;

        // when
        let entry = read_dir(&tmp_dir)?.next().unwrap()?;
        let filename = entry.file_name().to_str().unwrap().to_string();

        // then
        assert_eq!(filename, entry.filename());

        Ok(())
    }
}
