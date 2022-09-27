//! This is a specific implementation of a [`crate::use_cases::persistence`] mod.
//!
//! It uses just regular File System primitives to provide persistence.
use crate::helpers::PathRefExt;
use crate::result::PersistenceErr;
use crate::use_cases::persistence::DataPersistence;

use std::fs::{self, create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, instrument};

pub struct FsPersistence;

impl DataPersistence for FsPersistence {
    #[instrument(skip(self, buf))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), PersistenceErr> {
        let parent_dir = uri.parent_path();
        if !parent_dir.exists() {
            create_dir_all(parent_dir)?;
            // NOTE: this is needed because when file creation happens immediately after directory
            // creation, then the file creation event is not yet registered by filesystem watching
            // library
            thread::sleep(Duration::from_secs(1)); // allow to start watching for new directory
        }
        let mut file = File::create(uri)?;
        file.write_all(buf)?;
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Option<Vec<u8>>, PersistenceErr> {
        if !uri.exists() {
            debug!("uri: '{}' don't exist", uri.display());
            return Ok(None);
        }
        debug!("returning file under '{}'", uri.display());
        Ok(Some(fs::read(uri)?))
    }
}
