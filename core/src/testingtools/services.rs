// TODO: Split this module into smaller ones

use crate::entities::document::DocDetails;
use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::CipherErr;
use crate::result::{FsErr, IndexerErr, SearchErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::cipher::{
    Cipher, CipherRead, CipherReadStrategy, CipherStrategy, CipherWrite, CipherWriteStrategy,
};
use crate::use_cases::fs::{Filesystem, Fs};
use crate::use_cases::repository::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite, SearchResult,
};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, instrument};

pub type WorkingFs = NoOpFs;

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    pub fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
        let (read_tx, read_spy) = pipe();
        let (write_tx, write_spy) = pipe();

        (
            RepoSpies::new(read_spy, write_spy),
            Box::new(Self {
                read: TrackedRepoRead::create(repo.read(), read_tx),
                write: TrackedRepoWrite::create(repo.write(), write_tx),
            }),
        )
    }
}

impl Repository for TrackedRepo {
    fn read(&self) -> RepoRead {
        self.read.clone()
    }

    fn write(&self) -> RepoWrite {
        self.write.clone()
    }
}

pub struct TrackedRepoRead {
    read: RepoRead,
    #[allow(unused)]
    tx: Tx,
}

impl TrackedRepoRead {
    fn create(read: RepoRead, tx: Tx) -> RepoRead {
        Arc::new(Self { read, tx })
    }
}

impl RepositoryRead for TrackedRepoRead {
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr> {
        self.read.search(user, q)
    }

    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.read.all_docs(user)
    }
}

pub struct TrackedRepoWrite {
    write: RepoWrite,
    tx: Tx,
}

impl TrackedRepoWrite {
    fn create(write: RepoWrite, tx: Tx) -> RepoWrite {
        Arc::new(Self { write, tx })
    }
}

impl RepositoryWrite for TrackedRepoWrite {
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        debug!("before indexing");
        self.write.index(docs_details)?;
        self.tx.signal();
        debug!("after indexing");
        Ok(())
    }
}

pub struct RepoSpies {
    #[allow(unused)]
    read_spy: Spy,
    write_spy: Spy,
}

impl RepoSpies {
    fn new(read_spy: Spy, write_spy: Spy) -> Self {
        Self {
            read_spy,
            write_spy,
        }
    }

    #[allow(unused)]
    pub fn read(&self) -> &Spy {
        &self.read_spy
    }

    pub fn write(&self) -> &Spy {
        &self.write_spy
    }
}

pub struct CipherSpies {
    #[allow(unused)]
    read_spy: Spy,
    write_spy: Spy,
}

impl CipherSpies {
    fn new(read_spy: Spy, write_spy: Spy) -> Self {
        Self {
            read_spy,
            write_spy,
        }
    }

    #[allow(unused)]
    pub fn read(&self) -> &Spy {
        &self.read_spy
    }

    pub fn write(&self) -> &Spy {
        &self.write_spy
    }
}

pub struct FailingLoadFs;

impl FailingLoadFs {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Filesystem for FailingLoadFs {
    fn save(&self, _uri: PathBuf, _buf: &[u8]) -> Result<(), FsErr> {
        Ok(())
    }

    fn load(&self, _uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        Err(FsErr::Test)
    }

    fn rm_file(&self, _path: &SafePathBuf) -> Result<(), FsErr> {
        Ok(())
    }

    fn mv_file(&self, _from: &SafePathBuf, _to: &Path) -> Result<(), FsErr> {
        Ok(())
    }
}

pub struct TrackedCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl TrackedCipher {
    pub fn wrap(cipher: &Cipher) -> (CipherSpies, Cipher) {
        let (read_tx, read_spy) = pipe();
        let (write_tx, write_spy) = pipe();

        (
            CipherSpies::new(read_spy, write_spy),
            Box::new(Self {
                read: TrackedCipherRead::create(cipher.read(), read_tx),
                write: TrackedCipherWrite::create(cipher.write(), write_tx),
            }),
        )
    }
}

impl CipherStrategy for TrackedCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

pub struct TrackedCipherRead {
    read: CipherRead,
    #[allow(unused)]
    tx: Tx,
}

impl TrackedCipherRead {
    fn create(read: CipherRead, tx: Tx) -> CipherRead {
        Arc::new(Self { read, tx })
    }
}

impl CipherReadStrategy for TrackedCipherRead {
    fn decrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        self.read.decrypt(src_buf)
    }
}

pub struct TrackedCipherWrite {
    write: CipherWrite,
    tx: Tx,
}

impl TrackedCipherWrite {
    fn create(write: CipherWrite, tx: Tx) -> CipherWrite {
        Arc::new(Self { write, tx })
    }
}

impl CipherWriteStrategy for TrackedCipherWrite {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        debug!("before encrypting");
        self.tx.signal();
        let res = self.write.encrypt(src_buf)?;
        debug!("after encryption");
        Ok(res)
    }
}
pub struct FailingCipher {
    read: CipherRead,
    write: CipherWrite,
}

impl FailingCipher {
    pub fn create() -> Cipher {
        Box::new(Self {
            read: FailingCipherRead::new(),
            write: FailingCipherWrite::new(),
        })
    }
}

impl CipherStrategy for FailingCipher {
    fn read(&self) -> CipherRead {
        self.read.clone()
    }

    fn write(&self) -> CipherWrite {
        self.write.clone()
    }
}

pub struct FailingCipherRead;

impl FailingCipherRead {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherReadStrategy for FailingCipherRead {
    fn decrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

pub struct FailingCipherWrite;

impl FailingCipherWrite {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl CipherWriteStrategy for FailingCipherWrite {
    fn encrypt(&self, _src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        Err(CipherErr::Chacha(chacha20poly1305::Error))
    }
}

// TODO: Implement tracking for the rest of methods in other services
pub struct TrackedFs {
    fs: Fs,
    load_tx: Tx,
    save_tx: Tx,
    rm_file_tx: Tx,
    mv_file_tx: Tx,
}

impl TrackedFs {
    pub fn wrap(fs: Fs) -> (FsSpies, Fs) {
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
        self.fs.save(uri, buf)?;
        self.save_tx.signal();
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        let res = self.fs.load(uri)?;
        self.load_tx.signal();
        Ok(res)
    }

    #[instrument(skip(self))]
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        self.fs.rm_file(path)?;
        self.rm_file_tx.signal();
        Ok(())
    }

    #[instrument(skip(self))]
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr> {
        self.fs.mv_file(from, to)?;
        self.mv_file_tx.signal();
        Ok(())
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

pub struct NoOpFs;

impl NoOpFs {
    pub fn new() -> Arc<Self> {
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
