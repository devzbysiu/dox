use crate::use_cases::bus::{BusEvent, EventSubscriber};

use anyhow::{anyhow, Result};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub trait SubscriberExt {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent>;
}

impl SubscriberExt for EventSubscriber {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent> {
        let (done_tx, done_rx) = channel();
        let handle = thread::spawn(move || -> Result<()> {
            let event = self.recv()?;
            done_tx.send(event)?;
            Ok(())
        });

        match done_rx.recv_timeout(timeout) {
            Ok(event) => {
                if let Err(e) = handle.join() {
                    panic!("failed to join thread: {:?}", e);
                }
                Ok(event)
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}
