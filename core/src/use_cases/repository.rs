use serde::Serialize;

use crate::entities::document::DocDetails;
use crate::result::Result;

pub trait Repository: Sync + Send {
    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()>;
    fn search(&self, q: String) -> Result<SearchResult>;
    fn all_documents(&self) -> Result<SearchResult>;
}

#[derive(Debug, Serialize)]
pub struct SearchResult;
