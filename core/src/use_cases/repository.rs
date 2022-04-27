//! Abstraction for indexing and searching documents.
use std::collections::HashSet;

use crate::entities::document::DocDetails;
use crate::result::Result;

use dyn_clonable::clonable;
use serde::Serialize;

/// Allows to index and search documents.
#[clonable]
pub trait Repository: Clone + Sync + Send {
    /// Indexes documents.
    fn index(&self, docs_details: &[DocDetails]) -> Result<()>;
    /// Returns list of documents mathing passed query.
    fn search(&self, q: String) -> Result<SearchResult>;
    /// Returns list of all indexed documents.
    fn all_documents(&self) -> Result<SearchResult>;
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
