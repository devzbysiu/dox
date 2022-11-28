use std::sync::Arc;

use crate::result::CipherErr;
use crate::use_cases::cipher::{
    Cipher, CipherRead, CipherReadStrategy, CipherStrategy, CipherWrite, CipherWriteStrategy,
};

use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, XNonce};
use once_cell::sync::OnceCell;

pub struct Chacha20Poly1305Cipher {
    read: CipherRead,
    write: CipherWrite,
}

impl Chacha20Poly1305Cipher {
    pub fn create() -> Cipher {
        Box::new(Self {
            read: Arc::new(Chacha20Poly1305Read),
            write: Arc::new(Chacha20Poly1305Write),
        })
    }
}

impl CipherStrategy for Chacha20Poly1305Cipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

pub struct Chacha20Poly1305Read;

impl CipherReadStrategy for Chacha20Poly1305Read {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        decrypt(src_buf, key(), nonce())
    }
}

pub struct Chacha20Poly1305Write;

impl CipherWriteStrategy for Chacha20Poly1305Write {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
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

fn encrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>, CipherErr> {
    let cipher = XChaCha20Poly1305::new(key);
    Ok(cipher.encrypt(nonce, src_buf)?)
}

fn decrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>, CipherErr> {
    let cipher = XChaCha20Poly1305::new(key);
    Ok(cipher.decrypt(nonce, src_buf)?)
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use claim::assert_ok;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;

    #[test]
    fn encryption_return_success() {
        // given
        let cipher = Chacha20Poly1305Cipher::create();
        let buf: String = Paragraph(1..2).fake();

        // when
        let res = cipher.write().encrypt(buf.as_bytes());

        // then
        assert_ok!(res);
    }

    #[test]
    fn cipher_write_uses_chacha20poly1305_encryption() -> Result<()> {
        // given
        let cipher = Chacha20Poly1305Cipher::create();
        let buf: String = Paragraph(1..2).fake();
        let chacha = XChaCha20Poly1305::new(key());
        let expected = chacha.encrypt(nonce(), buf.as_bytes())?;

        // when
        let encrypted = cipher.write().encrypt(buf.as_bytes())?;

        // then
        assert_eq!(encrypted, expected);

        Ok(())
    }

    #[test]
    fn cipher_read_uses_chacha20poly1305_encryption() -> Result<()> {
        // given
        let cipher = Chacha20Poly1305Cipher::create();
        let buf: String = Paragraph(1..2).fake();
        let chacha = XChaCha20Poly1305::new(key());
        let encrypted = chacha.encrypt(nonce(), buf.as_bytes())?;

        // when
        let decrypted = cipher.read().decrypt(&encrypted)?;

        // then
        assert_eq!(decrypted, buf.as_bytes());

        Ok(())
    }

    #[test]
    fn cipher_read_can_read_output_of_cipher_write() -> Result<()> {
        // given
        let cipher = Chacha20Poly1305Cipher::create();
        let buf: String = Paragraph(1..2).fake();
        let encrypted = cipher.write().encrypt(buf.as_bytes())?;

        // when
        let decrypted = cipher.read().decrypt(&encrypted)?;

        // then
        assert_eq!(decrypted, buf.as_bytes());

        Ok(())
    }
}
