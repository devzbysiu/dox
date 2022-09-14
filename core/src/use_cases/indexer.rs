use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::repository::RepoWrite;

use std::thread;
use tracing::{debug, instrument, warn};

pub struct Indexer<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Indexer<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, repository))]
    pub fn run(&self, repository: RepoWrite) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::TextExtracted(user, doc_details) => {
                        repository.index(user, &doc_details)?;
                        publ.send(BusEvent::Indexed(doc_details))?;
                    }
                    e => debug!("event not supported in indexer: {}", e.to_string()),
                }
            }
        });
    }
}
