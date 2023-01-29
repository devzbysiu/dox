//! Abstraction representing the location on a medium, containing set of documents.
//!
//! This is medium agnostic, so the particular way the documents are read is part of the
//! implementation.
use crate::entities::extension::Ext;
use crate::helpers::PathRefExt;
use crate::result::GeneralErr;

use fake::{Dummy, Fake};
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

// TODO: Is this always true? Think about the case when `SafePathBuf` us used in
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
        self.0.ext()
    }

    // unused is allowed because this fn is used in #[cfg(not(test))]
    // see: https://github.com/rust-lang/rust-analyzer/issues/3860
    #[allow(unused)]
    pub fn parent(&self) -> &Path {
        self.0.parent().unwrap() // can unwrap because it's checked during construction
    }
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
