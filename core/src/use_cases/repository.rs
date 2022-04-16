use crate::entities::document::DocDetails;
use crate::result::Result;

pub trait Repository: Sync + Send {
    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()>;
}
