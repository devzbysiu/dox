use crate::helpers::PathRefExt;

use std::path::Path;

#[derive(Debug, PartialOrd, Ord, Eq, PartialEq)]
pub struct DocDetails {
    pub filename: String,
    pub body: String,
    pub thumbnail: String,
}

impl DocDetails {
    pub fn new<P: AsRef<Path>, S: Into<String>>(path: P, body: S, thumbnail: S) -> Self {
        Self {
            filename: path.filename(),
            body: body.into(),
            thumbnail: thumbnail.into(),
        }
    }
}
