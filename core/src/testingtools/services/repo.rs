use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::entities::user::User;
use crate::result::{BusErr, IndexerErr, SearchErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::repository::{
    AppState, AppStateReader, AppStateWriter, SearchResult, State, StateReader, StateWriter,
};

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::instrument;

pub fn tracked(repo: &State) -> (RepoSpies, State) {
    TrackedRepo::wrap(repo)
}

pub struct TrackedRepo {
    read: StateReader,
    write: StateWriter,
}

impl TrackedRepo {
    fn wrap(repo: &State) -> (RepoSpies, State) {
        let (search_tx, search_spy) = pipe();
        let (all_docs_tx, all_docs_spy) = pipe();

        let (index_tx, index_spy) = pipe();
        let (delete_tx, delete_spy) = pipe();

        (
            RepoSpies::new(search_spy, all_docs_spy, index_spy, delete_spy),
            Box::new(Self {
                read: TrackedRepoRead::create(repo.reader(), search_tx, all_docs_tx),
                write: TrackedRepoWrite::create(repo.writer(), index_tx, delete_tx),
            }),
        )
    }
}

impl AppState for TrackedRepo {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

pub struct TrackedRepoRead {
    read: StateReader,
    #[allow(unused)]
    search_tx: Tx,
    #[allow(unused)]
    all_docs_tx: Tx,
}

impl TrackedRepoRead {
    fn create(read: StateReader, search_tx: Tx, all_docs_tx: Tx) -> StateReader {
        Arc::new(Self {
            read,
            search_tx,
            all_docs_tx,
        })
    }
}

impl AppStateReader for TrackedRepoRead {
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr> {
        self.read.search(user, q)
    }

    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.read.all_docs(user)
    }
}

pub struct TrackedRepoWrite {
    write: StateWriter,
    index_tx: Tx,
    delete_tx: Tx,
}

impl TrackedRepoWrite {
    fn create(write: StateWriter, index_tx: Tx, delete_tx: Tx) -> StateWriter {
        Arc::new(Self {
            write,
            index_tx,
            delete_tx,
        })
    }
}

impl AppStateWriter for TrackedRepoWrite {
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

pub fn working() -> State {
    WorkingRepo::make()
}

struct WorkingRepo {
    read: StateReader,
    write: StateWriter,
}

impl WorkingRepo {
    fn make() -> State {
        Box::new(Self {
            read: WorkingRepoRead::new(),
            write: WorkingRepoWrite::new(),
        })
    }
}

impl AppState for WorkingRepo {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

struct WorkingRepoRead;

impl WorkingRepoRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for WorkingRepoRead {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }
}

struct WorkingRepoWrite;

impl WorkingRepoWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for WorkingRepoWrite {
    fn index(&self, _docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        Ok(())
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        Ok(())
    }
}

pub fn failing() -> State {
    FailingRepo::make()
}

struct FailingRepo {
    read: StateReader,
    write: StateWriter,
}

impl FailingRepo {
    fn make() -> State {
        Box::new(Self {
            read: FailingRepoRead::new(),
            write: FailingRepoWrite::new(),
        })
    }
}

impl AppState for FailingRepo {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

struct FailingRepoRead;

impl FailingRepoRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for FailingRepoRead {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }
}

struct FailingRepoWrite;

impl FailingRepoWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for FailingRepoWrite {
    fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
        Err(IndexerErr::Bus(BusErr::Generic(anyhow!("error"))))
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        unimplemented!()
    }
}

pub fn noop() -> State {
    NoOpRepo::make()
}

struct NoOpRepo {
    read: StateReader,
    write: StateWriter,
}

impl NoOpRepo {
    fn make() -> State {
        Box::new(Self {
            read: NoOpRepoRead::new(),
            write: NoOpRepoWrite::new(),
        })
    }
}

impl AppState for NoOpRepo {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

struct NoOpRepoRead;

impl NoOpRepoRead {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for NoOpRepoRead {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        // nothing to do
        Ok(Vec::new().into())
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        // nothing to do
        Ok(Vec::new().into())
    }
}

struct NoOpRepoWrite;

impl NoOpRepoWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for NoOpRepoWrite {
    fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
        // nothing to do here
        Ok(())
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        // nothing to do here
        Ok(())
    }
}
