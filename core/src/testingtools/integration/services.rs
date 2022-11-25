use crate::entities::document::DocDetails;
use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::{FsErr, IndexerErr, SearchErr};
use crate::testingtools::unit::Spy;
use crate::use_cases::fs::Filesystem;
use crate::use_cases::repository::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite, SearchResult,
};

use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use tracing::debug;

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
                read: TrackedRead::create(repo.read(), read_tx),
                write: TrackedWrite::create(repo.write(), write_tx),
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

pub struct TrackedRead {
    read: RepoRead,
    #[allow(unused)]
    tx: Mutex<Sender<()>>,
}

impl TrackedRead {
    fn create(read: RepoRead, tx: Mutex<Sender<()>>) -> RepoRead {
        Arc::new(Self { read, tx })
    }
}

impl RepositoryRead for TrackedRead {
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr> {
        self.read.search(user, q)
    }

    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.read.all_docs(user)
    }
}

pub struct TrackedWrite {
    write: RepoWrite,
    tx: Mutex<Sender<()>>,
}

impl TrackedWrite {
    fn create(write: RepoWrite, tx: Mutex<Sender<()>>) -> RepoWrite {
        Arc::new(Self { write, tx })
    }
}

impl RepositoryWrite for TrackedWrite {
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

pub struct FailingLoadFs;

impl FailingLoadFs {
    pub fn new() -> Arc<Self> {
        Arc::new(FailingLoadFs)
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
}
