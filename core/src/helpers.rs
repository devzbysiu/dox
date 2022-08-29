use crate::entities::extension::Ext;
use crate::result::Result;

use std::fs::DirEntry;
use std::path::Path;

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}

pub trait PathRefExt {
    fn ext(&self) -> Ext;
    fn str(&self) -> &str;
    fn string(&self) -> String;
    fn filestem(&self) -> String;
    fn filename(&self) -> String;
    fn first_filename(&self) -> Result<String>;
    fn parent_name(&self) -> String;
    fn parent_path(&self) -> &Path;
    fn rel_path(&self) -> String;
    fn rel_stem(&self) -> String;
}

impl<T: AsRef<Path>> PathRefExt for T {
    fn ext(&self) -> Ext {
        Ext::from(self.as_ref().extension().unwrap().to_str().unwrap())
    }

    fn str(&self) -> &str {
        self.as_ref().to_str().expect("path is not utf8")
    }

    fn string(&self) -> String {
        self.as_ref().to_string_lossy().to_string()
    }

    fn filestem(&self) -> String {
        let path = self.as_ref();
        path.file_stem()
            .unwrap_or_else(|| panic!("path '{}' does not have a filestem", path.display()))
            .to_string_lossy()
            .to_string()
    }

    fn filename(&self) -> String {
        let path = self.as_ref();
        path.file_name()
            .unwrap_or_else(|| panic!("path '{}' does not have a filename", path.display()))
            .to_string_lossy()
            .to_string()
    }

    fn first_filename(&self) -> Result<String> {
        Ok(self.as_ref().read_dir()?.next().unwrap()?.filename())
    }

    fn parent_name(&self) -> String {
        self.parent_path().filename()
    }

    fn parent_path(&self) -> &Path {
        self.as_ref().parent().expect("failed to get parent dir")
    }

    fn rel_path(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filename())
    }

    fn rel_stem(&self) -> String {
        format!("{}/{}", self.parent_name(), self.filestem())
    }
}

// TODO: think about moving it somewhere more important
pub mod cipher {
    use crate::result::Result;

    use chacha20poly1305::aead::{Aead, OsRng};
    use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, XNonce};
    use once_cell::sync::OnceCell;

    pub fn key() -> &'static Key {
        static KEY_INSTANCE: OnceCell<Key> = OnceCell::new();

        KEY_INSTANCE.get_or_init(|| XChaCha20Poly1305::generate_key(&mut OsRng))
    }

    pub fn nonce() -> &'static XNonce {
        static NONCE_INSTANCE: OnceCell<XNonce> = OnceCell::new();

        NONCE_INSTANCE.get_or_init(|| XChaCha20Poly1305::generate_nonce(&mut OsRng))
    }

    pub fn encrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(key);
        let encrypted = cipher.encrypt(nonce, src_buf)?;
        Ok(encrypted)
    }

    pub fn decrypt(src_buf: &[u8], key: &Key, nonce: &XNonce) -> Result<Vec<u8>> {
        let cipher = XChaCha20Poly1305::new(key);
        let decrypted = cipher.decrypt(nonce, src_buf)?;
        Ok(decrypted)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::fs::{read_dir, File};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_filename_in_dir_entry_ext() -> Result<()> {
        // given
        let tmp_dir = tempdir()?;
        File::create(tmp_dir.path().join("test-file"))?;

        // when
        let entry = read_dir(&tmp_dir)?.next().unwrap()?;
        let filename = entry.file_name().to_str().unwrap().to_string();

        // then
        assert_eq!(filename, entry.filename());

        Ok(())
    }

    #[test]
    fn test_ext_for_supported_extensions() {
        // given
        let test_cases = vec![
            ("png", Ext::Png),
            ("jpg", Ext::Jpg),
            ("jpeg", Ext::Jpg),
            ("webp", Ext::Webp),
            ("pdf", Ext::Pdf),
        ];

        for test_case in test_cases {
            // when
            let path = format!("/some-path/here.{}", test_case.0);
            let path = Path::new(&path);

            // then
            assert_eq!(path.ext(), test_case.1);
        }
    }

    #[test]
    #[should_panic(expected = "failed to create extension from 'txt'")]
    fn test_ext_for_not_supported_extensions() {
        // given
        let path = Path::new("/not-supported/extension.txt");

        // then
        let _ = path.ext(); // should panic
    }

    #[test]
    fn test_string_in_path_ref_ext() {
        // given
        let path = Path::new("/some-path/here");

        // when
        let string = path.string();

        // then
        assert_eq!(path.to_string_lossy().to_string(), string);
    }

    #[test]
    fn test_str_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let result = path.str();

        // then
        assert_eq!(path.to_str().unwrap(), result);
    }

    #[test]
    fn test_filestem_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let filestem = path.filestem();

        // then
        assert_eq!(
            path.file_stem().unwrap().to_string_lossy().to_string(),
            filestem
        );
    }

    #[test]
    fn test_filename_in_path_ref_ext() {
        // given
        let path = PathBuf::from("/some-path/here");

        // when
        let filename = path.filename();

        // then
        assert_eq!(
            path.file_name().unwrap().to_string_lossy().to_string(),
            filename
        );
    }
}
