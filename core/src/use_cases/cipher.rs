use crate::result::CipherErr;

use std::sync::Arc;

// TODO: Document this

pub type Cipher = Box<dyn CipherStrategy>;
pub type CipherRead = Arc<dyn CipherReadStrategy>;
pub type CipherWrite = Arc<dyn CipherWriteStrategy>;

pub trait CipherStrategy: Send {
    fn read(&self) -> CipherRead;
    fn write(&self) -> CipherWrite;
}

pub trait CipherReadStrategy: Sync + Send {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}

pub trait CipherWriteStrategy: Sync + Send {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}
