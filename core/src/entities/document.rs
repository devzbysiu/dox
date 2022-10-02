//! Abstraction of the document data used to index the document.
use crate::entities::user::User;
use crate::helpers::PathRefExt;

use std::path::Path;

/// Data of the document.
///
/// The values in this structure are used to do the indexing.
#[derive(Debug, PartialOrd, Clone, Ord, Eq, PartialEq)]
pub struct DocDetails {
    pub filename: String,
    pub body: String,
    pub thumbnail: String,
    pub user: User,
}

impl DocDetails {
    pub fn new<P: AsRef<Path>, S: Into<String>>(
        user: User,
        path: P,
        body: S,
        thumbnail: S,
    ) -> Self {
        Self {
            filename: path.filename(),
            body: body.into(),
            thumbnail: thumbnail.into(),
            user,
        }
    }
}
