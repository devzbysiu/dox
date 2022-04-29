//! Abstract interfaces for sending and receiving events.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::location::Location;
use crate::result::Result;

use std::fmt::Debug;

/// Allows receiving [`Event`]s.
///
/// This can be send across threads.
pub trait Input: Send + Debug {
    /// Receives one [`Event`].
    fn recv(&self) -> Result<ExternalEvent>;
}

/// Allows sending [`Event`]s.
///
/// This can be send across threads.
pub trait Output: Send + Debug {
    /// Sends one [`Location`].
    fn send(&self, event: ExternalEvent) -> Result<()>;
}

/// Represents events happening in the system.
#[derive(Debug, Clone)]
#[allow(unused)]
pub enum ExternalEvent {
    /// Represents new documents appearing in the system.
    NewDocs(Location),
}

#[derive(Debug, Clone)]
pub enum InternalEvent {
    DocumentReady,
}
