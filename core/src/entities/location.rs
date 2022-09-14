//! Abstraction representing the location on a medium, containing set of documents.
//!
//! This is medium agnostic, so the particular way the documents are read is part of the
//! implementation.
use crate::entities::extension::Ext;
use crate::helpers::PathRefExt;

use std::path::PathBuf;

/// Represents abstraction of the location on some medium.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// Represents local disk as a medium.
    FS(Vec<PathBuf>),
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
