//! Abstraction representing the location on a medium, containing set of documents.
//!
//! This is medium agnostic, so the particular way the documents are read is part of the
//! implementation.
use crate::entities::extension::Ext;
use crate::helpers::PathRefExt;

use fake::{Dummy, Fake};
use std::fmt::Display;
use std::path::{Path, PathBuf};

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
    pub fn extension(&self) -> Ext {
        let Location::FS(paths) = self;
        paths
            .get(0)
            .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
            .ext()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Dummy)]
pub struct SafePathBuf(PathBuf);

impl SafePathBuf {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        assert!(path.exists(), "Can't create not existing path '{:?}'", path);
        assert!(path.parent().is_some(), "Can't use '{:?}' as path", path);
        Self(path.into())
    }

    pub fn ext(&self) -> Ext {
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
