use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::entities::user::User;
use crate::result::{IndexerErr, SearchErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::repository::{
    Repo, RepoRead, RepoWrite, Repository, RepositoryRead, RepositoryWrite, SearchResult,
};

use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

pub fn tracked_repo(repo: &Repo) -> (RepoSpies, Repo) {
    TrackedRepo::wrap(repo)
}

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
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

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        unimplemented!()
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
