use std::path::PathBuf;

use rocket::{http::Status, response::Responder};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoxErr {
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
    OcrExtract(#[from] leptess::tesseract::TessInitError),

    #[error("error when setting image: '{0}'")]
    Image(#[from] leptess::leptonica::PixError),

    #[error("error when converting to utf8: '{0}'")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("error when extracting text from pdf: '{0}'")]
    PdfExtract(#[from] pdf_extract::OutputError),
}

pub type Result<T> = std::result::Result<T, DoxErr>;

// TODO: make sure that's the right way to go
impl<'r, 'o> Responder<'r, 'o> for DoxErr
where
    'o: 'r,
{
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}
