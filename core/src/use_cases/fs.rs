//! Abstraction used to save received documents.
use crate::entities::location::SafePathBuf;
use crate::result::FsErr;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type Fs = Arc<dyn Filesystem>;

// TODO: Update docs
/// Abstracts the process of saving document.
///
/// The actual implementation can be as simple as saving file on disk, saving file in
/// the cloud, or keeping it in memory. The implemention details are left for the implementer.
pub trait Filesystem: Sync + Send {
    /// Saves buffer passed as second argument under path specified as first argument.
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr>;

    /// Loads the file under first argument.
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr>;

    /// Removes file specified by the `path` argument.
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr>;

    /// Moves file from `from` path to `to` path.
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr>;
}
