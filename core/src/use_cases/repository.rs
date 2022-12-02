//! Abstraction for indexing and searching documents.
use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::entities::user::User;
use crate::result::{IndexerErr, SearchErr};

use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;

pub type Repo = Box<dyn Repository>;
pub type RepoRead = Arc<dyn RepositoryRead>;
pub type RepoWrite = Arc<dyn RepositoryWrite>;

pub trait Repository: Send {
    fn read(&self) -> RepoRead;
    fn write(&self) -> RepoWrite;
}

/// Allows to search and list all indexed documents .
pub trait RepositoryRead: Sync + Send {
    /// Returns list of documents mathing passed query.
    fn search(&self, user: User, q: String) -> Result<SearchResult, SearchErr>;
    /// Returns list of all indexed documents.
    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr>;
}

/// Allows to index documents.
pub trait RepositoryWrite: Sync + Send {
    /// Indexes documents.
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr>;

    fn delete(&self, loc: &Location) -> Result<(), IndexerErr>;
}

/// Holds list of basic document details.
#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct SearchResult {
    entries: HashSet<SearchEntry>,
}

impl From<Vec<SearchEntry>> for SearchResult {
    fn from(entries: Vec<SearchEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

/// Basic document details.
#[derive(Debug, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SearchEntry {
    filename: String,
    thumbnail: String,
}

impl SearchEntry {
    pub fn new((filename, thumbnail): (String, String)) -> Self {
        Self {
            filename,
            thumbnail,
        }
    }
}
