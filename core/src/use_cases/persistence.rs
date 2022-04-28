//! Abstraction used to store received documents.
use crate::result::Result;

use std::path::PathBuf;

/// Abstracts the process of saving document.
///
/// The actual implementation can be as simple as saving file on disk, saving file in
/// the cloud, or keeping it in memory. The implemention details are left for the implementer.
pub trait Persistence: Sync + Send {
    /// Saves buffer passed as second argument under path specified as first argument.
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()>;
}
