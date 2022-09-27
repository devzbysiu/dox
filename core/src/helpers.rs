use crate::entities::extension::Ext;
use crate::result::HelperErr;

use retry::delay::Fixed;
use retry::{retry, OperationResult};
use rocket::local::blocking::LocalResponse;
use rocket::{http::Status, local::blocking::Client};
use std::fs::DirEntry;
use std::io::Read;
use std::path::Path;
use tracing::{debug, instrument};

pub trait LocalResponseExt {
    fn read_body(&mut self) -> Result<String, HelperErr>;
}

impl LocalResponseExt for LocalResponse<'_> {
    #[instrument]
    fn read_body(&mut self) -> Result<String, HelperErr> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;
        let res = String::from_utf8(buffer)?;
        debug!("read the whole buffer: '{}'", res);
        Ok(res)
    }
}

pub trait ClientExt {
    fn read_entries(&self, endpoint: &str) -> Result<(String, Status), HelperErr>;
}

impl ClientExt for Client {
    fn read_entries(&self, endpoint: &str) -> Result<(String, Status), HelperErr> {
        Ok(retry(Fixed::from_millis(1000).take(60), || {
            let mut r = self.get(endpoint).dispatch();
            match r.read_body() {
                Ok(b) if b == r#"{"entries":[]}"# => OperationResult::Retry(("Empty", r.status())),
                Ok(b) if b.is_empty() => OperationResult::Retry(("Empty", r.status())),
                Ok(b) => OperationResult::Ok((b, r.status())),
                _ => OperationResult::Err(("Failed to fetch body", Status::InternalServerError)),
            }
        })
        .unwrap())
    }
}

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}

pub trait PathRefExt {
    fn ext(&self) -> Ext;
    fn str(&self) -> &str;
    fn string(&self) -> String;
    fn filestem(&self) -> String;
    fn filename(&self) -> String;
    fn first_filename(&self) -> Result<String, HelperErr>;
    fn parent_name(&self) -> String;
    fn parent_path(&self) -> &Path;
    fn rel_path(&self) -> String;
    fn rel_stem(&self) -> String;
    fn is_in_user_dir(&self) -> bool;
}

impl<T: AsRef<Path>> PathRefExt for T {
    fn ext(&self) -> Ext {
        Ext::from(self.as_ref().extension().unwrap().to_str().unwrap())
    }

    fn str(&self) -> &str {
        self.as_ref().to_str().expect("path is not utf8")
    }

    fn string(&self) -> String {
        self.as_ref().to_string_lossy().to_string()
    }

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

    fn first_filename(&self) -> Result<String, HelperErr> {
        Ok(self.as_ref().read_dir()?.next().unwrap()?.filename())
    }

    fn parent_name(&self) -> String {
        self.parent_path().filename()
    }

    fn parent_path(&self) -> &Path {
        self.as_ref().parent().expect("failed to get parent dir")
    }

    fn rel_path(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filename())
    }

    fn rel_stem(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filestem())
    }

    fn is_in_user_dir(&self) -> bool {
        // TODO: Add email validation and confirmation that the path is utf8 encoded
        let dir_name = self.parent_name();
        base64::decode(dir_name).is_ok()
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
    fn test_string_in_path_ref_ext() {
        // given
        let path = Path::new("/some-path/here");

        // when
        let string = path.string();

        // then
        assert_eq!(path.to_string_lossy().to_string(), string);
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

    #[test]
    fn test_filename_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let filename = path.filename();

        // then
        assert_eq!(
            path.file_name().unwrap().to_string_lossy().to_string(),
            filename
        );
    }
}
