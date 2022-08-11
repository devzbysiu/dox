//! This is aconcrete implementation of a [`crate::use_cases::persistence`] mod.
//!
//! It uses just regular File System primitives to provide persistence.
use crate::result::Result;
use crate::use_cases::persistence::DataPersistence;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct FsPersistence;

impl DataPersistence for FsPersistence {
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()> {
        let mut file = File::create(uri)?;
        file.write_all(buf)?;
        Ok(())
    }
}
