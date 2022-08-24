//! Abstraction used to save received documents.
use crate::result::Result;

use std::fs::File;
use std::path::PathBuf;

pub type Persistence = Box<dyn DataPersistence>;

/// Abstracts the process of saving document.
///
/// The actual implementation can be as simple as saving file on disk, saving file in
/// the cloud, or keeping it in memory. The implemention details are left for the implementer.
pub trait DataPersistence: Sync + Send {
    /// Saves buffer passed as second argument under path specified as first argument.
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()>;

    /// Loads the file under first argument.
    fn load(&self, uri: PathBuf) -> Result<Option<File>>;
}
