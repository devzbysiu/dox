//! Abstraction of the document data used to index the document.
use crate::entities::file::{Filename, Thumbnailname};
use crate::entities::user::User;

use fake::{Dummy, Fake};

/// Data of the document.
///
/// The values in this structure are used to do the indexing.
#[derive(Debug, PartialOrd, Clone, Ord, Eq, PartialEq, Dummy)]
pub struct DocDetails {
    pub filename: Filename,
    pub body: String,
    pub thumbnail: Thumbnailname,
    pub user: User,
}

impl DocDetails {
    pub fn new<S: Into<String>>(
        filename: Filename,
        body: S,
        thumbnail: Thumbnailname,
        user: User,
    ) -> Self {
        Self {
            filename,
            body: body.into(),
            thumbnail,
            user,
        }
    }
}
