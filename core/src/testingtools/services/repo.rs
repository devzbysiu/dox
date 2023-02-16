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

pub fn tracked(repo: &State) -> (StateSpies, State) {
    TrackedState::wrap(repo)
}

pub struct TrackedState {
    reader: StateReader,
    writer: StateWriter,
}

impl TrackedState {
    fn wrap(repo: &State) -> (StateSpies, State) {
        let (search_tx, search_spy) = pipe();
        let (all_docs_tx, all_docs_spy) = pipe();

        let (index_tx, index_spy) = pipe();
        let (delete_tx, delete_spy) = pipe();

        (
            StateSpies::new(search_spy, all_docs_spy, index_spy, delete_spy),
            Box::new(Self {
                reader: TrackedStateReader::create(repo.reader(), search_tx, all_docs_tx),
                writer: TrackedStateWriter::create(repo.writer(), index_tx, delete_tx),
            }),
        )
    }
}

impl AppState for TrackedState {
    fn reader(&self) -> StateReader {
        self.reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.writer.clone()
    }
}

pub struct TrackedStateReader {
    reader: StateReader,
    #[allow(unused)]
    search_tx: Tx,
    #[allow(unused)]
    all_docs_tx: Tx,
}

impl TrackedStateReader {
    fn create(reader: StateReader, search_tx: Tx, all_docs_tx: Tx) -> StateReader {
        Arc::new(Self {
            reader,
            search_tx,
            all_docs_tx,
        })
    }
}

impl AppStateReader for TrackedStateReader {
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr> {
        self.reader.search(user, q)
    }

    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.reader.all_docs(user)
    }
}

pub struct TrackedStateWriter {
    writer: StateWriter,
    index_tx: Tx,
    delete_tx: Tx,
}

impl TrackedStateWriter {
    fn create(writer: StateWriter, index_tx: Tx, delete_tx: Tx) -> StateWriter {
        Arc::new(Self {
            writer,
            index_tx,
            delete_tx,
        })
    }
}

impl AppStateWriter for TrackedStateWriter {
    #[instrument(skip(self))]
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        let res = self.writer.index(docs_details);
        self.index_tx.signal();
        res
    }

    #[instrument(skip(self))]
    fn delete(&self, loc: &Location) -> Result<(), IndexerErr> {
        let res = self.writer.delete(loc);
        self.delete_tx.signal();
        res
    }
}

pub struct StateSpies {
    #[allow(unused)]
    search_spy: Spy,
    #[allow(unused)]
    all_docs_spy: Spy,
    index_spy: Spy,
    delete_spy: Spy,
}

impl StateSpies {
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
    WorkingState::make()
}

struct WorkingState {
    reader: StateReader,
    writer: StateWriter,
}

impl WorkingState {
    fn make() -> State {
        Box::new(Self {
            reader: WorkingStateReader::new(),
            writer: WorkingStateWriter::new(),
        })
    }
}

impl AppState for WorkingState {
    fn reader(&self) -> StateReader {
        self.reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.writer.clone()
    }
}

struct WorkingStateReader;

impl WorkingStateReader {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for WorkingStateReader {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }
}

struct WorkingStateWriter;

impl WorkingStateWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for WorkingStateWriter {
    fn index(&self, _docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        Ok(())
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        Ok(())
    }
}

pub fn failing() -> State {
    FailingState::make()
}

struct FailingState {
    reader: StateReader,
    writer: StateWriter,
}

impl FailingState {
    fn make() -> State {
        Box::new(Self {
            reader: FailingStateReader::new(),
            writer: FailingStateWriter::new(),
        })
    }
}

impl AppState for FailingState {
    fn reader(&self) -> StateReader {
        self.reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.writer.clone()
    }
}

struct FailingStateReader;

impl FailingStateReader {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for FailingStateReader {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        Err(SearchErr::MissingIndex("error".into()))
    }
}

struct FailingStateWriter;

impl FailingStateWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for FailingStateWriter {
    fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
        Err(IndexerErr::Bus(BusErr::Generic(anyhow!("error"))))
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        unimplemented!()
    }
}

pub fn noop() -> State {
    NoOpState::make()
}

struct NoOpState {
    reader: StateReader,
    writer: StateWriter,
}

impl NoOpState {
    fn make() -> State {
        Box::new(Self {
            reader: NoOpStateReader::new(),
            writer: NoOpStateWriter::new(),
        })
    }
}

impl AppState for NoOpState {
    fn reader(&self) -> StateReader {
        self.reader.clone()
    }

    fn writer(&self) -> StateWriter {
        self.writer.clone()
    }
}

struct NoOpStateReader;

impl NoOpStateReader {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateReader for NoOpStateReader {
    fn search(&self, _user: User, _q: String) -> Result<SearchResult, SearchErr> {
        // nothing to do
        Ok(Vec::new().into())
    }

    fn all_docs(&self, _user: User) -> Result<SearchResult, SearchErr> {
        // nothing to do
        Ok(Vec::new().into())
    }
}

struct NoOpStateWriter;

impl NoOpStateWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl AppStateWriter for NoOpStateWriter {
    fn index(&self, _docs_details: &[DocDetails]) -> std::result::Result<(), IndexerErr> {
        // nothing to do here
        Ok(())
    }

    fn delete(&self, _loc: &Location) -> Result<(), IndexerErr> {
        // nothing to do here
        Ok(())
    }
}
