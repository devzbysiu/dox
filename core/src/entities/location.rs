//! Abstraction representing the location on a medium, containing set of documents.
//!
//! This is medium agnostic, so the particular way the documents are read is part of the
//! implementation.
use crate::entities::extension::Ext;
use crate::result::GeneralErr;

use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use fake::{Dummy, Fake};
use std::convert::TryFrom;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use tracing::instrument;

/// Represents abstraction of the location on some medium.
#[derive(Debug, Clone, PartialEq, Eq, Dummy)]
pub enum Location {
    /// Represents local disk as a medium.
    FS(Vec<SafePathBuf>),
}

impl Location {
    /// Provides extension of the file which location points to.
    ///
    /// This implementation assumes that all documents appearing in the system via one particular
    /// event, have the same extension. It's achieved by getting first path of the vector of paths
    /// and reading extension of this path.
    #[instrument(skip(self))]
    pub fn extension(&self) -> Result<Ext, GeneralErr> {
        let Location::FS(paths) = self;
        paths
            .get(0)
            .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
            .ext()
    }
}

// TODO: Is this `always present` true? Think about the case when `SafePathBuf` us used in
// [`std::fs::remove_file`] and similar.
/// Represents a path of resource which is always present.
///
/// It is not possible to create `SafePathBuf` poiting to not existing resource.
#[derive(Debug, Clone, PartialEq, Eq, Dummy)]
pub struct SafePathBuf(PathBuf);

impl SafePathBuf {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        {
            // NOTE: Tests are using mocked FS which does not manipulate files,
            // so for example moving a file won't actually move it thus we can't
            // check this during tests
            #[cfg(not(test))]
            assert!(
                path.exists(),
                "Can't create not existing path '{}'",
                path.display()
            );
        }
        assert!(
            path.parent().is_some(),
            "Can't use '{}' as path",
            path.display()
        );
        Self(path.into())
    }

    pub fn ext(&self) -> Result<Ext, GeneralErr> {
        let path = &self.0;
        match path.extension() {
            Some(ext) => Ok(Ext::try_from(ext.to_str().unwrap())?),
            None => Err(GeneralErr::InvalidExtension),
        }
    }

    // unused is allowed because this fn is used in #[cfg(not(test))]
    // see: https://github.com/rust-lang/rust-analyzer/issues/3860
    #[allow(unused)]
    pub fn parent(&self) -> &Path {
        self.0.parent().unwrap() // can unwrap because it's checked during construction
    }

    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    pub fn has_valid_ext(&self) -> bool {
        let path = &self.0;
        let Some(extension) = path.extension() else {
            return false;
        };
        match extension.to_str() {
            Some("png" | "jpg" | "jpeg" | "webp" | "pdf") => true,
            Some(_) | None => false,
        }
    }

    // TODO: Cover this with tests
    pub fn is_in_user_dir(&self) -> bool {
        // TODO: Add email validation and confirmation that the path is utf8 encoded
        let dir_name = self.parent_name();
        b64.decode(dir_name).is_ok()
    }

    // TODO: Cover this with tests
    pub fn rel_stem(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filestem())
    }

    // TODO: Cover this with tests
    pub fn rel_path(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filename())
    }

    // TODO: Cover this with tests
    pub fn parent_name(&self) -> String {
        filename_to_string(self.parent_path())
    }

    // TODO: Cover this with tests
    pub fn parent_path(&self) -> &Path {
        self.0.parent().expect("failed to get parent dir")
    }

    pub fn filestem(&self) -> String {
        let path = &self.0;
        path.file_stem()
            .unwrap_or_else(|| panic!("path '{}' does not have a filestem", path.display()))
            .to_string_lossy()
            .to_string()
    }

    // TODO: Cover this with tests
    pub fn filename(&self) -> String {
        filename_to_string(&self.0)
    }
}

fn filename_to_string<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    path.file_name()
        .unwrap_or_else(|| panic!("path '{}' does not have a filename", path.display()))
        .to_string_lossy()
        .to_string()
}

impl AsRef<PathBuf> for SafePathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for SafePathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<PathBuf> for SafePathBuf {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

impl From<&str> for SafePathBuf {
    fn from(path: &str) -> Self {
        Self::new(PathBuf::from(path))
    }
}

impl Display for SafePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use fake::{Fake, Faker};

    #[test]
    fn has_supported_extension_returns_true_for_supported_extensions() {
        // given
        let supported_extensions = vec!["png", "jpg", "jpeg", "webp", "pdf"];

        for test_case in supported_extensions {
            // when
            let path = format!("/some-path/here.{test_case}");
            let path = SafePathBuf::new(&path);

            // then
            assert!(path.has_valid_ext());
        }
    }

    #[test]
    fn false_is_returned_by_has_supported_extension_for_unsupported_extensions() {
        // given
        let unsupported_extension: String = Faker.fake(); // anything but supported ones

        // when
        let path = format!("/some-path/here.{unsupported_extension}");
        let path = SafePathBuf::new(path);

        // then
        assert!(!path.has_valid_ext());
    }

    #[test]
    fn test_filestem_in_path_ref_ext() {
        // given
        let path = SafePathBuf::from("/some-path/here");

        // when
        let filestem = path.filestem();

        // then
        assert_eq!(
            path.0.file_stem().unwrap().to_string_lossy().to_string(),
            filestem
        );
    }

    #[test]
    fn test_filename_in_path_ref_ext() {
        // given
        let path = SafePathBuf::from("/some-path/here");

        // when
        let filename = path.filename();

        // then
        assert_eq!(
            path.0.file_name().unwrap().to_string_lossy().to_string(),
            filename
        );
    }
}
