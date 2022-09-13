use crate::result::Result;

pub type CipherRead = Box<dyn CipherReadStrategy>;
pub type CipherWrite = Box<dyn CipherWriteStrategy>;

pub trait CipherReadStrategy: Sync + Send {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>>;
}

pub trait CipherWriteStrategy: Sync + Send {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>>;
}
