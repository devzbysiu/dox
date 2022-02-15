use std::path::PathBuf;

use rocket::response::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoxError {
    #[error("failed to create or write file: '{0}'")]
    Io(#[from] std::io::Error),

    #[error("failed to decode from base64: '{0}'")]
    Decode(#[from] base64::DecodeError),

    #[error("indexer failure: '{0}'")]
    Indexing(#[from] tantivy::TantivyError),

    #[error("error during query parsing: '{0}'")]
    Parsing(#[from] tantivy::query::QueryParserError),

    #[error("error from fs watcher: '{0}'")]
    FsWatcher(#[from] notify::Error),

    #[error("error when sending path through channel: '{0}'")]
    Send(#[from] std::sync::mpsc::SendError<PathBuf>),

    #[error("error when receiving list of paths through channel: '{0}'")]
    Recv(#[from] std::sync::mpsc::RecvError),

    #[error("error when deserializing from toml: '{0}'")]
    Toml(#[from] toml::de::Error),

    #[error("error when initializing LepTess: '{0}'")]
    Ocr(#[from] leptess::tesseract::TessInitError),

    #[error("error when setting image: '{0}'")]
    Image(#[from] leptess::leptonica::PixError),

    #[error("error when converting to utf8: '{0}'")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, DoxError>;

pub type RocketResult<T> = std::result::Result<T, Debug<DoxError>>;

pub fn to_debug_err(err: std::io::Error) -> Debug<DoxError> {
    Debug(DoxError::from(err))
}
