use std::path::PathBuf;

use crate::result::Result;

pub trait Persistence: Sync + Send {
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()>;
}
