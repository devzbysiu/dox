use crate::helpers::cipher::{self, key, nonce};
use crate::result::Result;
use crate::use_cases::cipher::{CipherRead, CipherReadStrategy, CipherWrite, CipherWriteStrategy};

pub struct Chacha20Poly1305Cipher;

impl Chacha20Poly1305Cipher {
    pub fn create() -> (CipherRead, CipherWrite) {
        (
            Box::new(Chacha20Poly1305Read),
            Box::new(Chacha20Poly1305Write),
        )
    }
}

pub struct Chacha20Poly1305Read;

impl CipherReadStrategy for Chacha20Poly1305Read {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>> {
        cipher::decrypt(src_buf, key(), nonce())
    }
}

pub struct Chacha20Poly1305Write;

impl CipherWriteStrategy for Chacha20Poly1305Write {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>> {
        cipher::encrypt(src_buf, key(), nonce())
    }
}
