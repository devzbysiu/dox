use crate::entities::location::SafePathBuf;
use crate::helpers::PathRefExt;
use crate::result::WrongNameErr;

use fake::{Dummy, Fake};
use serde::Deserialize;
use std::{fmt::Display, path::Path};

#[derive(Debug, Dummy, Clone, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Filename {
    filename: String,
}

impl Filename {
    pub fn new<S: Into<String>>(filename: S) -> Result<Self, WrongNameErr> {
        let filename = filename.into();
        if filename.is_empty() {
            Err(WrongNameErr::EmptyFilename)
        } else {
            Ok(Self { filename })
        }
    }

    pub fn has_supported_extension(&self) -> bool {
        let path = Path::new(&self.filename);
        let Some(extension) = path.extension() else {
            return false;
        };
        match extension.to_str() {
            Some("png" | "jpg" | "jpeg" | "webp" | "pdf") => true,
            Some(_) | None => false,
        }
    }

    pub fn stem(&self) -> String {
        Path::new(&self.filename).filename()
    }
}

impl Display for Filename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename)
    }
}

impl From<Filename> for String {
    fn from(value: Filename) -> Self {
        value.filename
    }
}

impl From<&SafePathBuf> for Filename {
    fn from(value: &SafePathBuf) -> Self {
        // TODO: Take care of this expect if makes sense
        Filename::new(value.filename()).expect("Failed to convert to Filename")
    }
}

#[derive(Debug, Dummy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Thumbnailname {
    thumbnail: String,
}

impl Thumbnailname {
    pub fn new<S: Into<String>>(thumbnail: S) -> Result<Self, WrongNameErr> {
        let thumbnail = thumbnail.into();
        if thumbnail.is_empty() {
            Err(WrongNameErr::EmptyThumbnailname)
        } else {
            Ok(Self { thumbnail })
        }
    }

    pub fn stem(&self) -> String {
        Path::new(&self.thumbnail).filename()
    }
}

impl From<&SafePathBuf> for Thumbnailname {
    fn from(value: &SafePathBuf) -> Self {
        // TODO: Take care of this expect if makes sense
        Thumbnailname::new(value.filename()).expect("Failed to convert to Filename")
    }
}

impl Display for Thumbnailname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.thumbnail)
    }
}
