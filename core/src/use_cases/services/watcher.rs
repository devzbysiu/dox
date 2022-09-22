use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent, EventBus};
use crate::use_cases::receiver::{DocsEvent, EventRecv};

use std::thread;
use tracing::{debug, trace, warn};

/// Watches for the event comming from [`Watcher`] and publishes appropriate event on the event bus.
///
/// It then spawns new thread in which it receives events from [`Watcher`]. If the event is
/// [`WatcherEvent::Created`], then [`Event::NewDocs`] is created out of it and published on the
/// bus.
#[derive(Debug)]
pub struct DocsWatcher {
    bus: EventBus,
}

impl DocsWatcher {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    pub fn run(&self, receiver: EventRecv) {
        debug!("spawning watching thread");
        let mut publ = self.bus.publisher();
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            loop {
                trace!("waiting for event from watcher");
                match receiver.recv() {
                    Ok(DocsEvent::Created(path)) => {
                        debug!("got create file event on path: '{:?}'", path);
                        publ.send(BusEvent::NewDocs(Location::FS(vec![path])))?;
                    }
                    Ok(e) => warn!("event not supported in Watcher: '{}'", e),
                    Err(e) => trace!("watcher error: {:?}", e),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::data_providers::bus::LocalBus;
    use crate::result::DoxErr;
    use crate::testutils::{mk_file, SubscriberExt};
    use crate::use_cases::bus::BusEvent;
    use crate::use_cases::receiver::EventReceiver;

    use anyhow::Result;
    use std::sync::mpsc::{channel, Receiver, RecvError};
    use std::time::Duration;

    #[test]
    fn created_docs_event_puts_new_docs_event_on_bus() -> Result<()> {
        // given
        init_tracing();
        let (tx, rx) = channel();
        let mock_event_receiver = Box::new(MockEventReceiver::new(rx));
        let bus = LocalBus::new()?;
        let new_file = mk_file("parent-dir".into(), "some-file.jpg".into())?;

        // when
        let watcher = DocsWatcher::new(bus.share());
        watcher.run(mock_event_receiver);
        tx.send(DocsEvent::Created(new_file.path.clone()))?;

        let sub = bus.subscriber();
        let event = sub.recv()?;

        // then
        assert_eq!(event, BusEvent::NewDocs(Location::FS(vec![new_file.path])));

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
        let watcher = DocsWatcher::new(bus.share());
        watcher.run(mock_event_receiver);
        tx.send(DocsEvent::Other).unwrap();
        let sub = bus.subscriber();

        // then
        sub.try_recv(Duration::from_secs(2)).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn errors_in_receiver_are_not_propagated_to_event_bus() {
        // given
        init_tracing();
        let erroneous_event_receiver = Box::new(ErroneousEventReceiver);
        let bus = LocalBus::new().unwrap();

        // when
        let watcher = DocsWatcher::new(bus.share());
        watcher.run(erroneous_event_receiver); // error ignored here
        let sub = bus.subscriber();

        // then
        sub.try_recv(Duration::from_secs(2)).unwrap(); // should panic
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

    struct ErroneousEventReceiver;

    impl EventReceiver for ErroneousEventReceiver {
        fn recv(&self) -> crate::result::Result<DocsEvent> {
            Err(DoxErr::Watcher(RecvError))
        }
    }
}
