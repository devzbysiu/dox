use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
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
                if let Event::TextExtracted(user, doc_details) = sub.recv()? {
                    repository.index(user, &doc_details)?;
                    publ.send(Event::Indexed(doc_details))?;
                } else {
                    debug!("event not supported here");
                }
            }
        });
    }
}
