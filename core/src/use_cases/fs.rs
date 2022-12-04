//! Abstraction used to work with documents.
use crate::entities::location::SafePathBuf;
use crate::result::FsErr;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type Fs = Arc<dyn Filesystem>;

/// Abstracts the process of manipulating a file resource.
///
/// The actual implementation can be as simple as saving file on disk, saving file in
/// the cloud, or keeping it in memory. The implemention details are left for the implementer.
pub trait Filesystem: Sync + Send {
    /// Saves buffer `buf` under path specified under `uri`.
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr>;

    /// Loads the file pointed by `uri`.
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr>;

    /// Removes file specified by the `uri` argument.
    fn rm_file(&self, uri: &SafePathBuf) -> Result<(), FsErr>;

    /// Moves file from `from` path to `to` path.
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr>;
}
