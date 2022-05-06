use crate::result::Result;
use crate::use_cases::bus::{Bus, Event};
use crate::use_cases::config::Config;
use crate::use_cases::repository::Repository;

use std::thread;
use tracing::log::debug;
use tracing::{instrument, warn};

pub struct Indexer;

impl Indexer {
    #[instrument(skip(bus, repository))]
    pub fn run(cfg: &Config, bus: &dyn Bus, repository: Box<dyn Repository>) {
        let sub = bus.subscriber();
        let mut publ = bus.publisher();
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
