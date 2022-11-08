use std::sync::Arc;

use crate::entities::location::SafePathBuf;
use crate::result::FsErr;

pub type Fs = Arc<dyn Filesystem>;

pub trait Filesystem: Send + Sync {
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr>;
}
