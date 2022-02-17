use crate::extractor::Ext;

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

pub trait ExtensionExt {
    fn ext(&self) -> Ext;
}

impl ExtensionExt for Path {
    fn ext(&self) -> Ext {
        Ext::from(self.extension().unwrap().to_str().unwrap())
    }
}
