use std::path::PathBuf;

use rocket::{http::Status, response::Responder};
use thiserror::Error;
use tungstenite::handshake::server::{NoCallback, ServerHandshake};
use tungstenite::handshake::HandshakeError;

// TODO: cleanup error messages and names
#[derive(Debug, Error)]
pub enum DoxErr {
    #[error("Invalid watched directory path: '{0}'")]
    InvalidWatchedDirPath(String),

    #[error("Invalid config path: '{0}'")]
    InvalidConfigPath(String),

    #[error("Failed to create or write file: '{0}'")]
    Io(#[from] std::io::Error),

    #[error("Failed to decode from base64: '{0}'")]
    Decode(#[from] base64::DecodeError),

    #[error("Failed to read index directory: '{0}'")]
    IndexDirectory(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error("Invalid index path: '{0}'")]
    InvalidIndexPath(String),

    #[error("Indexer failure: '{0}'")]
    Indexing(#[from] tantivy::TantivyError),

    #[error("Error during query parsing: '{0}'")]
    Parsing(#[from] tantivy::query::QueryParserError),

    #[error("Error from fs watcher: '{0}'")]
    FsWatcher(#[from] notify::Error),

    #[error("Error when sending path through channel: '{0}'")]
    Send(#[from] std::sync::mpsc::SendError<PathBuf>),

    #[error("Error when receiving list of paths through channel: '{0}'")]
    Recv(#[from] std::sync::mpsc::RecvError),

    #[error("Error when deserializing from TOML: '{0}'")]
    TomlDe(#[from] toml::de::Error),

    #[error("Error when initializing LepTess: '{0}'")]
    OcrExtract(#[from] leptess::tesseract::TessInitError),

    #[error("Error when setting image: '{0}'")]
    Image(#[from] leptess::leptonica::PixError),

    #[error("Error when converting to utf8: '{0}'")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Error when extracting text from PDF: '{0}'")]
    PdfExtract(#[from] pdf_extract::OutputError),

    #[error("Error while displaying prompt: '{0}'")]
    Prompt(#[from] inquire::error::InquireError),

    #[error("Error while serializing configuration: '{0}'")]
    TomlSe(#[from] toml::ser::Error),

    #[error("Error while creating PDF thumnail surface: '{0}'")]
    ThumbnailSurface(#[from] cairo::Error),

    #[error("Invalid thumbnails path: '{0}'")]
    InvalidThumbnailPath(String),

    #[error("Error writing thumbnail to file: '{0}'")]
    CarioIo(#[from] cairo::IoError),

    #[error("Error while creating poppler document for PDF thumbnail: '{0}'")]
    Poppler(#[from] cairo::glib::error::Error),

    #[error("Error while creating Websocket channel: '{0}'")]
    WebsocketConnection(#[from] HandshakeError<ServerHandshake<std::net::TcpStream, NoCallback>>),

    #[error("Error while writing websocket message: '{0}'")]
    Websocket(#[from] tungstenite::Error),

    #[error("Error while notifying about new docs: '{0}'")]
    Notifier(#[from] std::sync::mpsc::SendError<()>),

    #[error("Error while parsing to SocketAddrV4: '{0}'")]
    NotificationSocket(#[from] std::net::AddrParseError),
}

pub type Result<T> = std::result::Result<T, DoxErr>;

// TODO: make sure that's the right way to go
impl<'r, 'o: 'r> Responder<'r, 'o> for DoxErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}
