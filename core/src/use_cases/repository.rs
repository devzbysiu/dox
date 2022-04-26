//! Abstraction for indexing and searching documents.
use crate::entities::document::DocDetails;
use crate::result::Result;

use dyn_clonable::clonable;
use serde::Serialize;

/// Allows to index and search documents.
#[clonable]
pub trait Repository: Clone + Sync + Send {
    /// Indexes documents.
    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()>;
    /// Returns list of documents mathing passed query.
    fn search(&self, q: String) -> Result<SearchResult>;
    /// Returns list of all indexed documents.
    fn all_documents(&self) -> Result<SearchResult>;
}

/// Holds list of basic document details.
#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct SearchResult {
    entries: Vec<SearchEntry>,
}

impl SearchResult {
    pub fn new(entries: Vec<SearchEntry>) -> Self {
        Self { entries }
    }
}

/// Basic document details.
#[derive(Debug, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
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
