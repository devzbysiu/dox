//! Represents event bus used to coordinate components.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::result::Result;

use std::fmt::Debug;

/// Generic bus.
///
/// It allows to publish and subscribe to particular events in the system. Publishing can be done
/// either via [`Publisher`] or via [`Bus::send`] method.
pub trait Bus: Send + Debug {
    fn subscriber(&self) -> Box<dyn Subscriber>;

    fn publisher(&self) -> Box<dyn Publisher>;

    /// Publishes [`Event`] without the need to create [`Publisher`].
    fn send(&self, event: Event) -> Result<()>;
}

// allows to pass Box<dyn Bus> as &dyn Bus
impl<T: Bus + ?Sized> Bus for Box<T> {
    fn subscriber(&self) -> Box<dyn Subscriber> {
        (**self).subscriber()
    }

    fn publisher(&self) -> Box<dyn Publisher> {
        (**self).publisher()
    }

    fn send(&self, event: Event) -> Result<()> {
        (**self).send(event)
    }
}

/// Represents abstraction for receiving events.
pub trait Subscriber: Send {
    fn recv(&self) -> Result<Event>;
}

// TODO: Think about splitting events to internal and external. Currently, it's not possible to
// subscribe only to Internal or only to External events
/// Represents events happening in the system.
///
/// It describes both - internal and external events.
#[derive(Debug, Clone)]
pub enum Event {
    /// Represents new documents appearing in the system. External event.
    NewDocs(Location),

    /// Represents document finished indexing. Internal event.
    DocumentReady,

    /// Published when thumbnail generation is finished.
    ThumbnailMade,

    /// Published when text extraction is finished.
    TextExtracted(Vec<DocDetails>),
}

/// Represents abstraction for sending events.
pub trait Publisher: Send {
    fn send(&mut self, event: Event) -> Result<()>;
}
