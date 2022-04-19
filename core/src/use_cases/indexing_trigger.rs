use crate::result::Result;
use crate::use_cases::event::{Emitter, Event, Sink};

use std::thread;
use tracing::{error, instrument};

#[allow(unused)]
#[derive(Debug)]
pub struct IndexingTrigger {
    sink: Box<dyn Sink>,
    emitter: Box<dyn Emitter>,
}

#[allow(unused)]
impl IndexingTrigger {
    pub fn new(sink: Box<dyn Sink>, emitter: Box<dyn Emitter>) -> Self {
        Self { sink, emitter }
    }

    #[instrument(skip(self))]
    pub fn run(self) {
        thread::spawn(move || -> Result<()> {
            loop {
                match self.sink.recv() {
                    Ok(Event::NewDocs(location)) => self.emitter.send(location)?,
                    Err(e) => error!("receiving event error: {:?}", e),
                }
            }
        });
    }
}
