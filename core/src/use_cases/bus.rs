//! Represents external and internal events.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::location::Location;
use crate::result::Result;

use std::fmt::Debug;

pub trait Bus {
    fn subscriber(&self) -> Box<dyn Subscriber>;
    fn publisher(&self) -> Box<dyn Publisher>;
    fn publish(&self, event: &Event) -> Result<()>;
}

pub trait Subscriber {
    fn recv(&self) -> Result<Event>;
}

pub enum Event {
    Internal(InternalEvent),
    External(ExternalEvent),
}

pub trait Publisher {
    fn publish(&self, event: &Event) -> Result<()>;
}

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
