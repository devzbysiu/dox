use crate::result::Result;
use crate::use_cases::cipher::{CipherRead, CipherReadStrategy, CipherWrite, CipherWriteStrategy};

use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, XNonce};
use once_cell::sync::OnceCell;

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
        decrypt(src_buf, key(), nonce())
    }
}

pub struct Chacha20Poly1305Write;

impl CipherWriteStrategy for Chacha20Poly1305Write {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>> {
        encrypt(src_buf, key(), nonce())
    }
}

// TODO: what about server restarts?
fn key() -> &'static Key {
    static KEY_INSTANCE: OnceCell<Key> = OnceCell::new();
    KEY_INSTANCE.get_or_init(|| XChaCha20Poly1305::generate_key(&mut OsRng))
}

// TODO: what about server restarts?
fn nonce() -> &'static XNonce {
    static NONCE_INSTANCE: OnceCell<XNonce> = OnceCell::new();
    NONCE_INSTANCE.get_or_init(|| XChaCha20Poly1305::generate_nonce(&mut OsRng))
}

fn encrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key);
    Ok(cipher.encrypt(nonce, src_buf)?)
}

fn decrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key);
    Ok(cipher.decrypt(nonce, src_buf)?)
}
