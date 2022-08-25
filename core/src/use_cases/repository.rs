//! Abstraction for indexing and searching documents.
use crate::data_providers::server::User;
use crate::entities::document::DocDetails;
use crate::result::Result;

use serde::Serialize;
use std::collections::HashSet;

pub type RepoRead = Box<dyn RepositoryRead>;
pub type RepoWrite = Box<dyn RepositoryWrite>;

/// Allows to search and list all indexed documents .
pub trait RepositoryRead: Sync + Send {
    /// Returns list of documents mathing passed query.
    fn search(&self, user: User, q: String) -> Result<SearchResult>;
    /// Returns list of all indexed documents.
    fn all_documents(&self, user: User) -> Result<SearchResult>;
}

/// Allows to index documents.
pub trait RepositoryWrite: Sync + Send {
    /// Indexes documents.
    fn index(&self, user: User, docs_details: &[DocDetails]) -> Result<()>;
}

/// Holds list of basic document details.
#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct SearchResult {
    entries: HashSet<SearchEntry>,
}

impl SearchResult {
    pub fn from_vec(entries: Vec<SearchEntry>) -> Self {
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
