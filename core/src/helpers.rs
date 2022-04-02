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
    use std::fs::{read_dir, File};
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
    fn test_ext_for_supported_extensions() {
        // given
        let test_cases = vec![
            ("png", Ext::Png),
            ("jpg", Ext::Jpg),
            ("jpeg", Ext::Jpg),
            ("webp", Ext::Webp),
            ("pdf", Ext::Pdf),
        ];

        for test_case in test_cases {
            // when
            let path = format!("/some-path/here.{}", test_case.0);
            let path = Path::new(&path);

            // then
            assert_eq!(path.ext(), test_case.1);
        }
    }

    #[test]
    #[should_panic(expected = "failed to create extension from 'txt'")]
    fn test_ext_for_not_supported_extensions() {
        // given
        let path = Path::new("/not-supported/extension.txt");

        // then
        let _ = path.ext(); // should panic
    }

    #[test]
    fn test_string_in_path_ext() -> Result<()> {
        // given
        let path = Path::new("/not-supported/extension.txt");

        // when
        let string = path.string();

        // then
        assert_eq!(path.to_string_lossy().to_string(), string);

        Ok(())
    }

    #[test]
    fn test_str_in_path_buf_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let result = path.str();

        // then
        assert_eq!(path.to_str().unwrap(), result);
    }

    #[test]
    fn test_filestem_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let filestem = path.filestem();

        // then
        assert_eq!(
            path.file_stem().unwrap().to_string_lossy().to_string(),
            filestem
        );
    }
}
