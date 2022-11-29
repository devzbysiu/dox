use crate::entities::location::SafePathBuf;
use crate::result::FsErr;
use crate::use_cases::fs::{Filesystem, Fs};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tracing::debug;

pub mod integration;
pub mod unit;

pub fn index_dir_path() -> Result<TempDir> {
    debug!("creating index directory");
    Ok(tempfile::tempdir()?)
}

pub fn watched_dir_path() -> Result<TempDir> {
    debug!("creating watched directory");
    Ok(tempfile::tempdir()?)
}

pub fn thumbnails_dir_path() -> Result<TempDir> {
    debug!("creating thumbnails directory");
    Ok(tempfile::tempdir()?)
}

pub struct Spy {
    rx: Receiver<()>,
}

impl Spy {
    pub fn new(rx: Receiver<()>) -> Self {
        Self { rx }
    }

    pub fn method_called(&self) -> bool {
        self.rx.recv_timeout(Duration::from_secs(30)).is_ok()
    }
}

pub struct NoOpFs;

impl NoOpFs {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Filesystem for NoOpFs {
    fn save(&self, _uri: PathBuf, _buf: &[u8]) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }

    fn load(&self, _uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        // nothing to do
        Ok(Vec::new())
    }

    fn rm_file(&self, _path: &SafePathBuf) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }
}

pub struct FsSpy;

impl FsSpy {
    pub fn working() -> (Spy, Fs) {
        let (tx, rx) = channel();
        (Spy::new(rx), WorkingFs::new(tx))
    }
}

struct WorkingFs {
    tx: Mutex<Sender<()>>,
}

impl WorkingFs {
    fn new(tx: Sender<()>) -> Arc<Self> {
        Arc::new(Self { tx: Mutex::new(tx) })
    }
}

impl Filesystem for WorkingFs {
    fn save(&self, _uri: PathBuf, _buf: &[u8]) -> Result<(), FsErr> {
        unimplemented!()
    }

    fn load(&self, _uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        unimplemented!()
    }

    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        debug!("pretending to remove '{}'", path);
        self.tx
            .lock()
            .expect("poisoned mutex")
            .send(())
            .expect("failed to send message");
        Ok(())
    }
}
