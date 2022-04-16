use crate::entities::location::Location;
use crate::result::Result;

use std::fmt::Debug;

pub trait Sink: Sync + Send + Debug {
    fn recv(&self) -> Result<Event>;
}

pub trait Emitter: Sync + Send + Debug {
    fn send(&self, location: Location) -> Result<()>;
}

#[derive(Debug)]
pub enum Event {
    NewDocs(Location),
}
