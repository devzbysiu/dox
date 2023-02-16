//! Abstraction used to encrypt and decrypt data.
use crate::result::CipherErr;

use std::sync::Arc;

pub type Cipher = Box<dyn CipherStrategy>;
pub type CipherReader = Arc<dyn CipherReaderStrategy>;
pub type CipherWriter = Arc<dyn CipherWriterStrategy>;

/// Exposes tools for decrypting (`read`) and encrypting (`write`) data.
pub trait CipherStrategy: Send {
    fn reader(&self) -> CipherReader;
    fn writer(&self) -> CipherWriter;
}

/// Abstracts decrypting data.
pub trait CipherReaderStrategy: Sync + Send {
    /// Decrypts data passed in `buf` buffer.
    ///
    /// Returns `Vec` containing decrypted data.
    fn decrypt(&self, buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}

/// Abstracts encrypting data.
pub trait CipherWriterStrategy: Sync + Send {
    /// Encrypts data passed in `buf` buffer.
    ///
    /// Returns `Vec` containing encrypted data.
    fn encrypt(&self, buf: &[u8]) -> Result<Vec<u8>, CipherErr>;
}
