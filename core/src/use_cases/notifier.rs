//! Sends notification about new documents appearing in the system.
use crate::result::Result;

/// Abstraction for sending notification about new document. The receiving end could be a mobile
/// app, local process which does some postprocessing etc. The details are left for the
/// implementation.
pub trait Notifier: Send {
    /// Notifies that new document appeared in the system.
    ///
    /// The notification is sent after document has been preprocessed and indexing was triggered.
    /// The matter of waiting for the end of the document processing is left the implementation of
    /// the document preprocessing routine. See [`Indexer::run`](crate::use_cases::indexer::Indexer::run)
    fn notify(&self) -> Result<()>;
}
