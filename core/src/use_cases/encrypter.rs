use crate::entities::location::Location;
use crate::helpers::cipher::{self, key, nonce};
use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};

use std::fs;
use std::thread;
use tracing::{debug, instrument, warn};

pub struct Encrypter<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Encrypter<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self))]
    pub fn run(&self) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        // TODO: improve tracing of threads somehow. Currently, it's hard to debug because threads
        // do not appear as separate tracing's scopes
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    Event::EncryptionRequest(location) => {
                        let Location::FileSystem(paths) = location;
                        for path in paths {
                            let encrypted = cipher::encrypt(&fs::read(&path)?, key(), nonce())?;
                            fs::write(path, encrypted)?;
                        }
                        publ.send(Event::PipelineFinished)?;
                    }
                    // TODO: consider converting other if let parts to match to get the event and
                    // log it
                    e => debug!("event not supported in encrypter: {}", e.to_string()),
                }
            }
        });
    }
}
