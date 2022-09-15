use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::receiver::{DocsEvent, EventRecv};

use std::path::PathBuf;
use std::thread;
use tracing::{debug, trace, warn};

/// Watches for the event comming from [`Watcher`] and publishes appropriate event on the event bus.
///
/// It then spawns new thread in which it receives events from [`Watcher`]. If the event is
/// [`WatcherEvent::Created`], then [`Event::NewDocs`] is created out of it and published on the
/// bus.
#[derive(Debug)]
pub struct DocsWatcher<'a> {
    bus: &'a dyn Bus,
}

impl<'a> DocsWatcher<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    pub fn run(&self, receiver: EventRecv) {
        debug!("spawning watching thread");
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            loop {
                debug!("waiting for event from watcher");
                match receiver.recv() {
                    Ok(DocsEvent::Created(path)) => {
                        debug!("got create file event on path: '{}'", path.display());
                        publ.send(new_docs_event(path))?;
                    }
                    Ok(e) => warn!("this event is not supported: {:?}", e),
                    Err(e) => trace!("watcher error: {:?}", e),
                }
            }
        });
    }
}

fn new_docs_event(path: PathBuf) -> BusEvent {
    debug!("new doc appeared, creating NewDocs event");
    BusEvent::NewDocs(Location::FS(vec![path]))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::data_providers::bus::LocalBus;
    use crate::use_cases::bus::EventSubscriber;
    use crate::use_cases::receiver::EventReceiver;

    use anyhow::{anyhow, Result};
    use std::sync::mpsc::{channel, Receiver};
    use std::time::Duration;

    #[test]
    fn created_docs_event_puts_new_docs_event_on_bus() -> Result<()> {
        // given
        init_tracing();
        let (tx, rx) = channel();
        let mock_event_receiver = Box::new(MockEventReceiver::new(rx));
        let bus = LocalBus::new()?;

        // when
        let watcher = DocsWatcher::new(&bus);
        watcher.run(mock_event_receiver);
        tx.send(DocsEvent::Created("path".into()))?;

        let sub = bus.subscriber();
        let event = sub.recv()?;

        // then
        assert_eq!(event, BusEvent::NewDocs(Location::FS(vec!["path".into()])));

        Ok(())
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn other_docs_event_is_ignored() {
        // given
        init_tracing();
        let (tx, rx) = channel();
        let mock_event_receiver = Box::new(MockEventReceiver::new(rx));
        let bus = LocalBus::new().unwrap();

        // when
        let watcher = DocsWatcher::new(&bus);
        watcher.run(mock_event_receiver);
        tx.send(DocsEvent::Other).unwrap();
        let sub = bus.subscriber();

        // then
        sub.try_recv(Duration::from_secs(2)).unwrap(); // should panic
    }

    trait SubscriberExt {
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

    struct MockEventReceiver {
        rx: Receiver<DocsEvent>,
    }

    impl MockEventReceiver {
        fn new(rx: Receiver<DocsEvent>) -> Self {
            Self { rx }
        }
    }

    impl EventReceiver for MockEventReceiver {
        fn recv(&self) -> crate::result::Result<DocsEvent> {
            Ok(self.rx.recv()?)
        }
    }
}
