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
use tracing::instrument;

pub fn tracked_repo(repo: &Repo) -> (RepoSpies, Repo) {
    TrackedRepo::wrap(repo)
}

pub struct TrackedRepo {
    read: RepoRead,
    write: RepoWrite,
}

impl TrackedRepo {
    fn wrap(repo: &Repo) -> (RepoSpies, Repo) {
        let (search_tx, search_spy) = pipe();
        let (all_docs_tx, all_docs_spy) = pipe();

        let (index_tx, index_spy) = pipe();
        let (delete_tx, delete_spy) = pipe();

        (
            RepoSpies::new(search_spy, all_docs_spy, index_spy, delete_spy),
            Box::new(Self {
                read: TrackedRepoRead::create(repo.read(), search_tx, all_docs_tx),
                write: TrackedRepoWrite::create(repo.write(), index_tx, delete_tx),
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
    search_tx: Tx,
    #[allow(unused)]
    all_docs_tx: Tx,
}

impl TrackedRepoRead {
    fn create(read: RepoRead, search_tx: Tx, all_docs_tx: Tx) -> RepoRead {
        Arc::new(Self {
            read,
            search_tx,
            all_docs_tx,
        })
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
    index_tx: Tx,
    delete_tx: Tx,
}

impl TrackedRepoWrite {
    fn create(write: RepoWrite, index_tx: Tx, delete_tx: Tx) -> RepoWrite {
        Arc::new(Self {
            write,
            index_tx,
            delete_tx,
        })
    }
}

impl RepositoryWrite for TrackedRepoWrite {
    #[instrument(skip(self))]
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        let res = self.write.index(docs_details);
        self.index_tx.signal();
        res
    }

    #[instrument(skip(self))]
    fn delete(&self, loc: &Location) -> Result<(), IndexerErr> {
        let res = self.write.delete(loc);
        self.delete_tx.signal();
        res
    }
}

pub struct RepoSpies {
    #[allow(unused)]
    search_spy: Spy,
    #[allow(unused)]
    all_docs_spy: Spy,
    index_spy: Spy,
    delete_spy: Spy,
}

impl RepoSpies {
    fn new(search_spy: Spy, all_docs_spy: Spy, index_spy: Spy, delete_spy: Spy) -> Self {
        Self {
            search_spy,
            all_docs_spy,
            index_spy,
            delete_spy,
        }
    }

    #[allow(unused)]
    pub fn search_called(&self) -> bool {
        self.search_spy.method_called()
    }

    #[allow(unused)]
    pub fn all_docs_called(&self) -> bool {
        self.all_docs_spy.method_called()
    }

    pub fn index_called(&self) -> bool {
        self.index_spy.method_called()
    }

    pub fn delete_called(&self) -> bool {
        self.delete_spy.method_called()
    }
}
