//! Represents external and internal events.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::location::Location;

use std::fmt::Debug;

/// Represents external events happening in the system.
#[derive(Debug, Clone)]
pub enum ExternalEvent {
    /// Represents new documents appearing in the system.
    NewDocs(Location),
}

/// Represents internal core events.
#[derive(Debug, Clone)]
pub enum InternalEvent {
    /// Represents document finished indexing.
    DocumentReady,
}
