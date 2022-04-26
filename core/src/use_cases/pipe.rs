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
    fn recv(&self) -> Result<Event>;
}

/// Allows sending [`Event`]s.
///
/// This can be send across threads.
pub trait Output: Send + Debug {
    /// Sends one [`Location`].
    fn send(&self, event: Event) -> Result<()>;
}

/// Represents events happening in the system.
#[allow(unused)]
#[derive(Debug)]
pub enum Event {
    /// Represents new documents appearing in the system.
    NewDocs(Location),
}