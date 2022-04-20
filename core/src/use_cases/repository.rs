use crate::entities::document::DocDetails;
use crate::result::Result;

use serde::Serialize;

pub trait Repository: Sync + Send {
    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()>;
    fn search(&self, q: String) -> Result<SearchResult>;
    fn all_documents(&self) -> Result<SearchResult>;
}

#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct SearchResult {
    entries: Vec<SearchEntry>,
}

impl SearchResult {
    pub fn new(entries: Vec<SearchEntry>) -> Self {
        Self { entries }
    }
}

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


