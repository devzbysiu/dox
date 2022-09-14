use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::cipher::CipherWrite;

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

    #[instrument(skip(self, cipher))]
    pub fn run(&self, cipher: CipherWrite) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        // TODO: improve tracing of threads somehow. Currently, it's hard to debug because threads
        // do not appear as separate tracing's scopes
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    Event::EncryptionRequest(location) => {
                        debug!("encryption request: '{:?}', starting encryption", location);
                        let Location::FileSystem(paths) = location;
                        for path in paths {
                            let encrypted = cipher.encrypt(&fs::read(&path)?)?;
                            fs::write(path, encrypted)?;
                        }
                        debug!("encryption finished");
                        publ.send(Event::PipelineFinished)?;
                    }
                    e => debug!("event not supported in encrypter: {}", e.to_string()),
                }
            }
        });
    }
}
