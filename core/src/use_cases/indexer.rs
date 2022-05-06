use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::repository::RepositoryWrite;

use std::thread;
use tracing::log::debug;
use tracing::{instrument, warn};

pub struct Indexer<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Indexer<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, repository))]
    pub fn run(&self, repository: Box<dyn RepositoryWrite>) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            loop {
                if let Event::TextExtracted(doc_details) = sub.recv()? {
                    repository.index(&doc_details)?;
                    publ.send(Event::DocumentReady)?;
                } else {
                    debug!("event not supported here");
                }
            }
        });
    }
}
