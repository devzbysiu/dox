use crate::extractor::Ext;

use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}

pub trait PathExt {
    fn ext(&self) -> Ext;
    fn string(&self) -> String;
}

impl PathExt for Path {
    fn ext(&self) -> Ext {
        Ext::from(self.extension().unwrap().to_str().unwrap())
    }

    fn string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

pub trait PathBufExt {
    fn str(&self) -> &str;
}

impl PathBufExt for PathBuf {
    fn str(&self) -> &str {
        self.to_str().expect("path is not utf8")
    }
}
