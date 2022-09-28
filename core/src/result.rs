use rocket::{http::Status, response::Responder};
use std::io::ErrorKind::NotFound;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThumbnailReadErr {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("Failed to load thumbnail.")]
    LoadError(#[from] PersistenceErr),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ThumbnailReadErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(match self {
            Self::LoadError(PersistenceErr::IoError(e)) if e.kind() == NotFound => Status::NotFound,
            Self::UnexpectedError(_) | Self::LoadError(_) => Status::InternalServerError,
        })
    }
}

#[derive(Debug, Error)]
pub enum PersistenceErr {
    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum DocumentReadErr {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for DocumentReadErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}

#[derive(Debug, Error)]
pub enum CipherErr {
    #[error("Failed to decrypt")]
    ChachaError(#[from] chacha20poly1305::Error),
}

#[derive(Debug, Error)]
pub enum DocumentSaveErr {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for DocumentSaveErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}

#[derive(Debug, Error)]
pub enum SearchErr {
    #[error("Failed while searching for document.")]
    DocumentFetchError(#[from] tantivy::TantivyError),

    #[error("No index for user '{0}' found")]
    MissingIndex(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for SearchErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}

#[derive(Debug, Error)]
pub enum ExtractorErr {
    #[error("Error when converting to utf8.")]
    PdfExtractionError(#[from] pdf_extract::OutputError),

    #[error("Error when converting to utf8.")]
    UserConversionError(#[from] UserConvErr),

    #[error("Error when publishing bus event.")]
    PublisherError(#[from] BusErr),

    #[error("Error when initializing LepTess.")]
    OcrExtractError(#[from] leptess::tesseract::TessInitError),

    #[error("Error when setting the image.")]
    SettingImageError(#[from] leptess::leptonica::PixError),

    #[error("Error when converting to utf8.")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Failed to create threadpool.")]
    ThreadPoolError(#[from] rayon::ThreadPoolBuildError),
}

#[derive(Debug, Error)]
pub enum PreprocessorErr {
    #[error("Error when using bus.")]
    BusError(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),

    #[error("Failed to load PDF document.")]
    PopplerError(#[from] cairo::glib::error::Error),

    #[error("Failed to create image from PDF page.")]
    ThumbnailSurfaceError(#[from] cairo::Error),

    #[error("Failed to write PDF thumbnail to image surface.")]
    CarioIoError(#[from] cairo::IoError),

    #[error("Failed to create threadpool.")]
    ThreadPoolError(#[from] rayon::ThreadPoolBuildError),
}

#[derive(Debug, Error)]
pub enum UserConvErr {
    #[error("Failed to decode from base64.")]
    Decode(#[from] base64::DecodeError),

    #[error("Invalid utf characters.")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[cfg(not(test))]
    #[error("No authorization header found.")]
    MissingToken,

    #[cfg(not(test))]
    #[error("Missing 'email' field in IdToken.")]
    InvalidIdToken,

    #[cfg(not(test))]
    #[error("Token could not be verified by identity provider.")]
    TokenVerification,
}

#[derive(Debug, Error)]
pub enum BusErr {
    #[error("Failed to create Eventador instance")]
    GenericError(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when publishing bus event.")]
    PublisherError(#[from] BusErr),
}

#[derive(Debug, Error)]
pub enum EncrypterErr {
    #[error("Error when using bus.")]
    BusError(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),

    #[error("Failed to encrypt data.")]
    CipherError(#[from] CipherErr),

    #[error("Failed to create threadpool.")]
    ThreadPoolError(#[from] rayon::ThreadPoolBuildError),
}

#[derive(Debug, Error)]
pub enum EventReceiverErr {
    #[error("Failed to create watcher.")]
    CreationError(#[from] notify::Error),

    #[error("Failed to receive event from watcher.")]
    ReceiveError(#[from] std::sync::mpsc::RecvError),
}

#[derive(Debug, Error)]
pub enum RepositoryErr {
    #[error("Failed to create repository.")]
    CreationError(#[from] notify::Error),

    #[error("Invalid index path: '{0}'")]
    InvalidIndexPath(String),

    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum IndexerErr {
    #[error("Failed to create threadpool.")]
    ThreadPoolError(#[from] rayon::ThreadPoolBuildError),

    #[error("Error when using bus.")]
    BusError(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),

    #[error("Failed to read index directory.")]
    OpenIndexDirectoryError(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error("Failed to access index directory.")]
    IndexDirectoryError(#[from] tantivy::TantivyError),
}

#[derive(Debug, Error)]
pub enum ConfigurationErr {
    #[error("Invalid index path: '{0}'")]
    InvalidIndexPath(String),

    #[error("Invalid thumbnails path: '{0}'")]
    InvalidThumbnailPath(String),

    #[error("Failed to make IO operation.")]
    IoError(#[from] std::io::Error),

    #[error("Invalid watched directory: '{0}'")]
    InvalidWatchedDirPath(String),

    #[error("Failed to deserialize configuration.")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Failed to serialize configuration.")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Invalid config path: '{0}'")]
    InvalidConfigPath(String),
}

#[derive(Debug, Error)]
pub enum PromptErr {
    #[error("Failed to display prompt.")]
    Prompt(#[from] inquire::error::InquireError),
}

#[derive(Debug, Error)]
pub enum SetupErr {
    #[error("Failed to run encrypter.")]
    EncrypterError(#[from] EncrypterErr),

    #[error("Failed to run watcher.")]
    WatcherError(#[from] WatcherErr),

    #[error("Failed to create receiver.")]
    EventReceiverError(#[from] EventReceiverErr),

    #[error("Failed to create repository.")]
    RepositoryError(#[from] RepositoryErr),

    #[error("Failed to run indexer.")]
    IndexerError(#[from] IndexerErr),

    #[error("Failed to get configuration.")]
    ConfigurationError(#[from] ConfigurationErr),

    #[error("Failed to launch Rocket.")]
    RocketErr(#[from] rocket::Error),
}
