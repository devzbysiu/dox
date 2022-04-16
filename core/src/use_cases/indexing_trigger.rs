use crate::result::Result;
use crate::use_cases::event::{Event, EventReceiver, EventTransmitter};

use std::thread;
use tracing::error;

pub struct IndexingTrigger {
    event_receiver: Box<dyn EventReceiver>,
    event_transmitter: Box<dyn EventTransmitter>,
}

impl IndexingTrigger {
    fn run(self) {
        thread::spawn(move || -> Result<()> {
            loop {
                match self.event_receiver.recv() {
                    Ok(Event::NewData(location)) => self.event_transmitter.send(location)?,
                    Err(e) => error!("receiving event error: {:?}", e),
                }
            }
        });
    }
}
