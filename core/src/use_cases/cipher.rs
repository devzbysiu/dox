//! Abstraction used to encrypt end decrypt data.
use crate::result::CipherErr;

use std::sync::Arc;

pub type Cipher = Box<dyn CipherStrategy>;
pub type CipherRead = Arc<dyn CipherReadStrategy>;
pub type CipherWrite = Arc<dyn CipherWriteStrategy>;

/// Exposes tools for decrypting (`read`) and encrypting (`write`) data.
pub trait CipherStrategy: Send {
    fn read(&self) -> CipherRead;
    fn write(&self) -> CipherWrite;
}

/// Abstracts decrypting data.
pub trait CipherReadStrategy: Sync + Send {
    /// Decrypts data passed in `buf` buffer.
    ///
    /// Returns `Vec` containing decrypted data.
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}

/// Abstracts encrypting data.
pub trait CipherWriteStrategy: Sync + Send {
    /// Encrypts data passed in `buf` buffer.
    ///
    /// Returns `Vec` containing encrypted data.
    fn encrypt(&self, buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}
