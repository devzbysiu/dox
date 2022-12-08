use crate::entities::location::SafePathBuf;
use crate::result::FsErr;
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::fs::{Filesystem, Fs};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::instrument;

pub fn failing() -> Fs {
    FailingFs::make()
}

pub struct FailingFs;

impl FailingFs {
    fn make() -> Fs {
        Arc::new(Self)
    }
}

impl Filesystem for FailingFs {
    fn save(&self, _uri: PathBuf, _buf: &[u8]) -> Result<(), FsErr> {
        Err(FsErr::Test)
    }

    fn load(&self, _uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        Err(FsErr::Test)
    }

    fn rm_file(&self, _path: &SafePathBuf) -> Result<(), FsErr> {
        Err(FsErr::Test)
    }

    fn mv_file(&self, _from: &SafePathBuf, _to: &Path) -> Result<(), FsErr> {
        Err(FsErr::Test)
    }
}

pub fn tracked(fs: Fs) -> (FsSpies, Fs) {
    TrackedFs::wrap(fs)
}

pub struct TrackedFs {
    fs: Fs,
    load_tx: Tx,
    save_tx: Tx,
    rm_file_tx: Tx,
    mv_file_tx: Tx,
}

impl TrackedFs {
    fn wrap(fs: Fs) -> (FsSpies, Fs) {
        let (load_tx, load_spy) = pipe();
        let (save_tx, save_spy) = pipe();
        let (rm_file_tx, rm_file_spy) = pipe();
        let (mv_file_tx, mv_file_spy) = pipe();

        (
            FsSpies::new(load_spy, save_spy, rm_file_spy, mv_file_spy),
            Arc::new(Self {
                fs,
                load_tx,
                save_tx,
                rm_file_tx,
                mv_file_tx,
            }),
        )
    }
}

impl Filesystem for TrackedFs {
    #[instrument(skip(self, buf))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        let res = self.fs.save(uri, buf);
        self.save_tx.signal();
        res
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        let res = self.fs.load(uri);
        self.load_tx.signal();
        res
    }

    #[instrument(skip(self))]
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        let res = self.fs.rm_file(path);
        self.rm_file_tx.signal();
        res
    }

    #[instrument(skip(self))]
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr> {
        let res = self.fs.mv_file(from, to);
        self.mv_file_tx.signal();
        res
    }
}

pub struct FsSpies {
    load_spy: Spy,
    save_spy: Spy,
    rm_file_spy: Spy,
    mv_file_spy: Spy,
}

impl FsSpies {
    fn new(load_spy: Spy, save_spy: Spy, rm_file_spy: Spy, mv_file_spy: Spy) -> Self {
        Self {
            load_spy,
            save_spy,
            rm_file_spy,
            mv_file_spy,
        }
    }

    #[allow(unused)]
    pub fn load_called(&self) -> bool {
        self.load_spy.method_called()
    }

    #[allow(unused)]
    pub fn save_called(&self) -> bool {
        self.save_spy.method_called()
    }

    pub fn rm_file_called(&self) -> bool {
        self.rm_file_spy.method_called()
    }

    pub fn mv_file_called(&self) -> bool {
        self.mv_file_spy.method_called()
    }
}

pub fn working() -> Fs {
    NoOpFs::new()
}

pub fn noop() -> Fs {
    NoOpFs::new()
}

pub struct NoOpFs;

impl NoOpFs {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Filesystem for NoOpFs {
    #[instrument(skip(self))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        // nothing to do
        Ok(Vec::new())
    }

    #[instrument(skip(self))]
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }

    #[instrument(skip(self))]
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr> {
        // nothing to do
        Ok(())
    }
}
