//! Abstract interfaces for sending and receiving events.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::location::Location;

use std::fmt::Debug;

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
