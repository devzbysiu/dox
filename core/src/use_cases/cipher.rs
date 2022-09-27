use std::sync::Arc;

use crate::result::{CipherErr, Result};

pub type CipherRead = Arc<dyn CipherReadStrategy>;
pub type CipherWrite = Arc<dyn CipherWriteStrategy>;

pub trait CipherReadStrategy: Sync + Send {
    fn decrypt(&self, src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr>;
}

pub trait CipherWriteStrategy: Sync + Send {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>>;
}
