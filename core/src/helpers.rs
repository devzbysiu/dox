use anyhow::{anyhow, Error};
use rocket::response::Debug;
use std::fs::DirEntry;

pub fn to_debug_err(err: std::io::Error) -> Debug<Error> {
    Debug(anyhow!("{}", err))
}

pub trait DirEntryExt {
    fn filename(&self) -> String;
}

impl DirEntryExt for DirEntry {
    fn filename(&self) -> String {
        self.file_name().to_str().unwrap().to_string()
    }
}
