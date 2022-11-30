use crate::entities::document::DocDetails;
use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::CipherErr;
use crate::result::{FsErr, IndexerErr, SearchErr};
use crate::testingtools::Spy;
use crate::use_cases::cipher::{
    Cipher, CipherRead, CipherReadStrategy, CipherStrategy, CipherWrite, CipherWriteStrategy,
};
use crate::use_cases::fs::{Filesystem, Fs};
use crate::use_cases::repository::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite, SearchResult,
};

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use tracing::{debug, instrument};

pub type WorkingFs = NoOpFs;

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    pub fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
        let (read_tx, read_rx) = channel();
        let (write_tx, write_rx) = channel();
        let read_tx = Mutex::new(read_tx);
        let write_tx = Mutex::new(write_tx);
        let read = Spy::new(read_rx);
        let write = Spy::new(write_rx);
        (
            RepoSpies { read, write },
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
    tx: Mutex<Sender<()>>,
}

impl TrackedRepoRead {
    fn create(read: RepoRead, tx: Mutex<Sender<()>>) -> RepoRead {
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

// TODO: Think about using decorator design pattern to limit number of implementations (like TrackedWrite is
// emulating a successfull repo, for failing repo you would have to implement TrackedFailingWrite instead of
// having just TrackedRepo and separately FailingWrite and SuccessfullWrite).
pub struct TrackedRepoWrite {
    write: RepoWrite,
    tx: Mutex<Sender<()>>,
}

impl TrackedRepoWrite {
    fn create(write: RepoWrite, tx: Mutex<Sender<()>>) -> RepoWrite {
        Arc::new(Self { write, tx })
    }
}

impl RepositoryWrite for TrackedRepoWrite {
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        debug!("before indexing");
        self.write.index(docs_details)?;
        let tx = self.tx.lock().expect("poisoned mutex");
        tx.send(()).expect("failed to send");
        debug!("after indexing");
        Ok(())
    }
}

pub struct RepoSpies {
    #[allow(unused)]
    read: Spy,
    write: Spy,
}

impl RepoSpies {
    #[allow(unused)]
    pub fn read(&self) -> &Spy {
        &self.read
    }

    pub fn write(&self) -> &Spy {
        &self.write
    }
}

pub struct CipherSpies {
    #[allow(unused)]
    read: Spy,
    write: Spy,
}

impl CipherSpies {
    #[allow(unused)]
    pub fn read(&self) -> &Spy {
        &self.read
    }

    pub fn write(&self) -> &Spy {
        &self.write
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
        let (read_tx, read_rx) = channel();
        let (write_tx, write_rx) = channel();
        let read_tx = Mutex::new(read_tx);
        let write_tx = Mutex::new(write_tx);
        let read = Spy::new(read_rx);
        let write = Spy::new(write_rx);
        (
            CipherSpies { read, write },
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
    tx: Mutex<Sender<()>>,
}

impl TrackedCipherRead {
    fn create(read: CipherRead, tx: Mutex<Sender<()>>) -> CipherRead {
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
    tx: Mutex<Sender<()>>,
}

impl TrackedCipherWrite {
    fn create(write: CipherWrite, tx: Mutex<Sender<()>>) -> CipherWrite {
        Arc::new(Self { write, tx })
    }
}

impl CipherWriteStrategy for TrackedCipherWrite {
    fn encrypt(&self, src_buf: &[u8]) -> Result<Vec<u8>, CipherErr> {
        debug!("before encrypting");
        let tx = self.tx.lock().expect("poisoned mutex");
        tx.send(()).expect("failed to send");
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

// TODO: Think about some abstraction for this `Mutex<Sender<()>>`
// TODO: Implement tracking for the rest of methods in other services
pub struct TrackedFs {
    fs: Fs,
    load_tx: Mutex<Sender<()>>,
    save_tx: Mutex<Sender<()>>,
    rm_file_tx: Mutex<Sender<()>>,
    mv_file_tx: Mutex<Sender<()>>,
}

impl TrackedFs {
    pub fn wrap(fs: Fs) -> (FsSpies, Fs) {
        let (load_tx, load_rx) = channel();
        let (save_tx, save_rx) = channel();
        let (rm_file_tx, rm_file_rx) = channel();
        let (mv_file_tx, mv_file_rx) = channel();

        let load_spy = Spy::new(load_rx);
        let save_spy = Spy::new(save_rx);
        let rm_file_spy = Spy::new(rm_file_rx);
        let mv_file_spy = Spy::new(mv_file_rx);

        let load_tx = Mutex::new(load_tx);
        let save_tx = Mutex::new(save_tx);
        let rm_file_tx = Mutex::new(rm_file_tx);
        let mv_file_tx = Mutex::new(mv_file_tx);

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

fn signal(tx: &Mutex<Sender<()>>) {
    let tx = tx.lock().expect("poisoned mutex");
    tx.send(()).expect("failed to send");
}

impl Filesystem for TrackedFs {
    #[instrument(skip(self, buf))]
    fn save(&self, uri: PathBuf, buf: &[u8]) -> Result<(), FsErr> {
        self.fs.save(uri, buf)?;
        signal(&self.save_tx);
        Ok(())
    }

    #[instrument(skip(self))]
    fn load(&self, uri: PathBuf) -> Result<Vec<u8>, FsErr> {
        let res = self.fs.load(uri)?;
        signal(&self.load_tx);
        Ok(res)
    }

    #[instrument(skip(self))]
    fn rm_file(&self, path: &SafePathBuf) -> Result<(), FsErr> {
        self.fs.rm_file(path)?;
        signal(&self.rm_file_tx);
        Ok(())
    }

    #[instrument(skip(self))]
    fn mv_file(&self, from: &SafePathBuf, to: &Path) -> Result<(), FsErr> {
        self.fs.mv_file(from, to)?;
        signal(&self.mv_file_tx);
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
    // TODO: Think about those Mutex<Receiver<()>>
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
