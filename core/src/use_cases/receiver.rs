use crate::result::Result;

use std::path::PathBuf;

pub type EventRecv = Box<dyn EventReceiver>;

pub trait EventReceiver: Send {
    fn recv(&self) -> Result<DocsEvent>;
}

#[derive(Debug)]
pub enum DocsEvent {
    Created(PathBuf),
    Other,
}
