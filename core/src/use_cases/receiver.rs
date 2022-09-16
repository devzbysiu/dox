use crate::entities::location::SafePathBuf;
use crate::result::Result;

pub type EventRecv = Box<dyn EventReceiver>;

pub trait EventReceiver: Send {
    fn recv(&self) -> Result<DocsEvent>;
}

#[derive(Debug)]
pub enum DocsEvent {
    Created(SafePathBuf),
    Other,
}
