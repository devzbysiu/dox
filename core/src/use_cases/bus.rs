//! Represents event bus used to coordinate components.
//!
//! The events represent new files of particular document, appearing in the system, which are going
//! to be indexed by dox' core.
use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::user::User;

use std::fmt::{Debug, Display};

pub type EventBus = Box<dyn Bus>;
pub type EventSubscriber = Box<dyn Subscriber>;
pub type EventPublisher = Box<dyn Publisher>;

/// Generic bus.
///
/// It allows to publish and subscribe to particular events in the system. Publishing can be done
/// either via [`Publisher`] or via [`Bus::send`] method.
pub trait Bus: Send + Sync + Debug {
    fn subscriber(&self) -> EventSubscriber;

    fn publisher(&self) -> EventPublisher;
}

// Allows to pass Box<dyn Bus> as &dyn Bus
impl<T: Bus + ?Sized> Bus for Box<T> {
    fn subscriber(&self) -> EventSubscriber {
        (**self).subscriber()
    }

    fn publisher(&self) -> EventPublisher {
        (**self).publisher()
    }
}

/// Represents abstraction for receiving events.
pub trait Subscriber: Send {
    fn recv(&self) -> Result<BusEvent>;
}

// TODO: Think about splitting events to internal and external. Currently, it's not possible to
// subscribe only to Internal or only to External events
/// Represents events happening in the system.
///
/// It describes both - internal and external events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusEvent {
    /// Represents new documents appearing in the system. External event.
    NewDocs(Location),

    /// Published when text extraction is finished.
    TextExtracted(User, Vec<DocDetails>),

    /// Published when thumbnail generation is finished.
    ThumbnailMade(Location),

    /// Represents document finished indexing. Internal event.
    Indexed(Vec<DocDetails>),

    /// Published when there is a time to encrypt the file.
    EncryptionRequest(Location),

    /// Published when document processing is finished.
    PipelineFinished,
}

impl Display for BusEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BusEvent::NewDocs(_) => "NewDocs",
                BusEvent::TextExtracted(_, _) => "TextExtracted",
                BusEvent::ThumbnailMade(_) => "ThumbnailMade",
                BusEvent::Indexed(_) => "Indexed",
                BusEvent::EncryptionRequest(_) => "EncryptionRequest",
                BusEvent::PipelineFinished => "PipelineFinished",
            }
        )
    }
}

/// Represents abstraction for sending events.
pub trait Publisher: Send {
    fn send(&mut self, event: BusEvent) -> Result<()>;
}
