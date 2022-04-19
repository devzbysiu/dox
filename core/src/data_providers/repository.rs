use crate::entities::document::DocDetails;
use crate::result::Result;
use crate::use_cases::repository::Repository;
use crate::use_cases::repository::SearchResult;

#[derive(Debug, Clone)]
pub struct TantivyRepository;

impl Repository for TantivyRepository {
    fn search(&self, q: String) -> Result<SearchResult> {
        unimplemented!()
    }

    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()> {
        unimplemented!()
    }

    fn all_documents(&self) -> Result<SearchResult> {
        unimplemented!()
    }
}
