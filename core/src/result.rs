use rocket::{http::Status, response::Responder};
use std::io::ErrorKind::NotFound;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneralErr {
    #[error("Extension not supported.")]
    InvalidExtension,
}

// NOTE: not sure if this is way to go. It's needed because the trait `TryFrom` uses `Infallible`
// and for some reason it's not possible to convert it to custom error type via thiserror.
// See https://github.com/dtolnay/thiserror/issues/62
impl From<std::convert::Infallible> for GeneralErr {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug, Error)]
pub enum ThumbnailReadErr {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error("Failed to load thumbnail.")]
    Load(#[from] FsErr),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ThumbnailReadErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(match self {
            Self::Load(FsErr::Io(e)) if e.kind() == NotFound => Status::NotFound,
            Self::Unexpected(_) | Self::Load(_) => Status::InternalServerError,
        })
    }
}

#[derive(Debug, Error)]
pub enum FsErr {
    // TODO: Should I add '{0}' everywhere?
    #[error("Failed to make IO operation: '{0}'.")]
    Io(#[from] std::io::Error),

    // TODO: Think if there is a better way
    #[cfg(test)]
    #[error("Purposefully faling IO operation")]
    Test,
}

#[derive(Debug, Error)]
pub enum DocumentReadErr {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error("Failed to load document.")]
    Load(#[from] FsErr),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for DocumentReadErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(match self {
            Self::Load(FsErr::Io(e)) if e.kind() == NotFound => Status::NotFound,
            Self::Unexpected(_) | Self::Load(_) => Status::InternalServerError,
        })
    }
}

#[derive(Debug, Error)]
pub enum CipherErr {
    #[error("Failed to decrypt")]
    Chacha(#[from] chacha20poly1305::Error),
}

#[derive(Debug, Error)]
pub enum DocumentSaveErr {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for DocumentSaveErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}

#[derive(Debug, Error)]
pub enum SearchErr {
    #[error("Failed while searching for document.")]
    DocumentFetch(#[from] tantivy::TantivyError),

    #[error("No index for user '{0}' found")]
    MissingIndex(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for SearchErr {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        Err(Status::new(500))
    }
}

#[derive(Debug, Error)]
pub enum ExtractorErr {
    #[error("Error when converting to utf8.")]
    PdfExtraction(#[from] pdf_extract::OutputError),

    #[error("Error when converting to utf8.")]
    UserConversion(#[from] UserConvErr),

    #[error("Error when publishing bus event.")]
    Publisher(#[from] BusErr),

    #[error("Error when initializing LepTess.")]
    OcrExtract(#[from] leptess::tesseract::TessInitError),

    #[error("Error when setting the image.")]
    SettingImage(#[from] leptess::leptonica::PixError),

    #[error("Error when converting to utf8.")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Failed to create threadpool.")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Invalid file extension")]
    InvalidExtension(#[from] GeneralErr),
}

#[derive(Debug, Error)]
pub enum PreprocessorErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),

    #[error("Failed to load PDF document.")]
    Poppler(#[from] cairo::glib::error::Error),

    #[error("Failed to create thumbnail from PDF page.")]
    ThumbnailSurface(#[from] cairo::Error),

    #[error("Failed to write PDF thumbnail to image surface.")]
    CarioIo(#[from] cairo::IoError),

    #[error("Failed to create threadpool.")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Invalid file extension")]
    InvalidExtension(#[from] GeneralErr),

    #[error("Failed to make filesystem operation")]
    Fs(#[from] FsErr),
}

#[derive(Debug, Error)]
pub enum MoverErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),

    #[error("Failed to load PDF document.")]
    Poppler(#[from] cairo::glib::error::Error),

    #[error("Failed to create thumbnail from PDF page.")]
    ThumbnailSurface(#[from] cairo::Error),

    #[error("Failed to write PDF thumbnail to image surface.")]
    CarioIo(#[from] cairo::IoError),

    #[error("Failed to create threadpool.")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Invalid file extension.")]
    InvalidExtension(#[from] GeneralErr),

    #[error("Failed to make filesystem operation: '{0}'.")]
    Fs(#[from] FsErr),
}

#[derive(Debug, Error)]
pub enum UserConvErr {
    #[error("Failed to decode from base64.")]
    Decode(#[from] base64::DecodeError),

    #[error("Invalid utf characters.")]
    Utf8(#[from] std::string::FromUtf8Error),

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
    Generic(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WatcherErr {
    #[error("Error when publishing bus event.")]
    Publisher(#[from] BusErr),
}

#[derive(Debug, Error)]
pub enum EncrypterErr {
    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),

    #[error("Failed to encrypt data.")]
    Cipher(#[from] CipherErr),

    #[error("Failed to create threadpool.")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Failed to encrypt all files.")]
    AllOrNothing,
}

#[derive(Debug, Error)]
pub enum EventReceiverErr {
    #[error("Failed to create watcher.")]
    Creation(#[from] notify::Error),

    #[error("Failed to receive event from watcher.")]
    Receive(#[from] std::sync::mpsc::RecvError),
}

#[derive(Debug, Error)]
pub enum RepositoryErr {
    #[error("Failed to create repository.")]
    Creation(#[from] notify::Error),

    #[error("Invalid index path: '{0}'")]
    InvalidIndexPath(String),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum IndexerErr {
    #[error("Failed to create threadpool.")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Error when using bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),

    #[error("Failed to read index directory.")]
    OpenIndexDirectory(#[from] tantivy::directory::error::OpenDirectoryError),

    #[error("Failed to access index directory.")]
    IndexDirectory(#[from] tantivy::TantivyError),

    #[error("Failed to make filesystem operation")]
    Fs(#[from] FsErr),
}

#[derive(Debug, Error)]
pub enum ConfigurationErr {
    #[error("Invalid index path: '{0}'")]
    InvalidIndexPath(String),

    #[error("Invalid thumbnails path: '{0}'")]
    InvalidThumbnailPath(String),

    #[error("Failed to make IO operation.")]
    Io(#[from] std::io::Error),

    #[error("Invalid watched directory: '{0}'")]
    InvalidWatchedDirPath(String),

    #[error("Failed to deserialize configuration.")]
    Deserialization(#[from] toml::de::Error),

    #[error("Failed to serialize configuration.")]
    Serialization(#[from] toml::ser::Error),

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
    #[error("Failed to create event bus.")]
    Bus(#[from] BusErr),

    #[error("Failed to run encrypter.")]
    Encrypter(#[from] EncrypterErr),

    #[error("Failed to run preprocessor.")]
    Preprocessor(#[from] PreprocessorErr),

    #[error("Failed to run extractor.")]
    Extractor(#[from] ExtractorErr),

    #[error("Failed to run watcher.")]
    Watcher(#[from] WatcherErr),

    #[error("Failed to create receiver.")]
    EventReceiver(#[from] EventReceiverErr),

    #[error("Failed to create repository.")]
    Repository(#[from] RepositoryErr),

    #[error("Failed to run document mover.")]
    Mover(#[from] MoverErr),

    #[error("Failed to run indexer.")]
    Indexer(#[from] IndexerErr),

    #[error("Failed to get configuration.")]
    Configuration(#[from] ConfigurationErr),

    #[error("Failed to launch Rocket.")]
    Rocket(#[from] rocket::Error),
}
