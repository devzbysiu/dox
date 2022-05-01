//! Represents event bus used to coordinate components.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::location::Location;
use crate::result::Result;

use std::fmt::Debug;

/// Generic bus.
///
/// It allows to publish and subscribe to particular events in the system. Publishing can be done
/// either via [`Publisher`] or via [`Bus::send`] method.
pub trait Bus: Send {
    /// Creates [`Subscriber`].
    fn subscriber(&self) -> Box<dyn Subscriber>;

    /// Creates [`Publisher`].
    fn publisher(&self) -> Box<dyn Publisher>;

    /// Publishes [`Event`] without the need to create [`Publisher`].
    fn send(&self, event: Event) -> Result<()>;
}

pub trait Subscriber: Send {
    fn recv(&self) -> Result<Event>;
}

#[derive(Debug, Clone)]
pub enum Event {
    Internal(InternalEvent),
    External(ExternalEvent),
}

pub trait Publisher: Send {
    fn publish(&mut self, event: Event) -> Result<()>;
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
