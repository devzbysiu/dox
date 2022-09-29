use crate::entities::location::SafePathBuf;
use crate::result::EventReceiverErr;

use std::fmt::Display;

pub type EventRecv = Box<dyn EventReceiver>;

pub trait EventReceiver: Send {
    fn recv(&self) -> Result<DocsEvent, EventReceiverErr>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum DocsEvent {
    Created(SafePathBuf),
    Other,
}

impl Display for DocsEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DocsEvent::Created(_) => "Created",
                DocsEvent::Other => "Other",
            }
        )
    }
}
