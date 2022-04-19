use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::event::Emitter;
use crate::use_cases::event::Event;
use crate::use_cases::event::Sink;

#[derive(Debug)]
pub struct FsSink;

impl FsSink {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sink for FsSink {
    fn recv(&self) -> Result<Event> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct DefaultEmitter;

impl DefaultEmitter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Emitter for DefaultEmitter {
    fn send(&self, location: Location) -> Result<()> {
        unimplemented!()
    }
}
