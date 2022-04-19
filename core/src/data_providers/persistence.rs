use crate::result::Result;
use crate::use_cases::persistence::Persistence;

use std::path::PathBuf;

pub struct FsPersistence;

impl Persistence for FsPersistence {
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<()> {
        unimplemented!()
    }
}
