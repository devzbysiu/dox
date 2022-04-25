use crate::result::Result;
use crate::use_cases::persistence::Persistence;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct FsPersistence;

impl Persistence for FsPersistence {
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()> {
        let mut file = File::create(uri)?;
        file.write_all(buf)?;
        Ok(())
    }
}
