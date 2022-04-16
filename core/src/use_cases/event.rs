use crate::result::Result;

use std::path::PathBuf;

pub trait EventReceiver: Sync + Send {
    fn recv(&self) -> Result<Event>;
}

pub trait EventTransmitter: Sync + Send {
    fn send(&self, location: Location) -> Result<()>;
}

#[derive(Debug)]
pub enum Event {
    NewData(Location),
}

#[derive(Debug)]
pub enum Location {
    FileSystem(PathBuf),
}
