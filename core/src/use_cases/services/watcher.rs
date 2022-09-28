use crate::entities::location::Location;
use crate::result::WatcherErr;
use crate::use_cases::bus::{BusEvent, EventBus};
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

    pub fn run(self, receiver: EventRecv) {
        debug!("spawning watching thread");
        thread::spawn(move || -> Result<(), WatcherErr> {
            debug!("watching thread spawned");
            let mut publ = self.bus.publisher();
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

    use crate::configuration::factories::event_bus;
    use crate::configuration::telemetry::init_tracing;
    use crate::result::EventReceiverErr;
    use crate::testingtools::{mk_file, SubscriberExt};
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
        let bus = event_bus()?;
        let new_file = mk_file("parent-dir".into(), "some-file.jpg".into())?;
        let watcher = DocsWatcher::new(bus.clone());
        watcher.run(mock_event_receiver);
        let sub = bus.subscriber();

        // when
        tx.send(DocsEvent::Created(new_file.path.clone()))?;

        // then
        assert_eq!(
            sub.recv()?,
            BusEvent::NewDocs(Location::FS(vec![new_file.path]))
        );

        Ok(())
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn other_docs_event_is_ignored() {
        // given
        init_tracing();
        let (tx, rx) = channel();
        let mock_event_receiver = Box::new(MockEventReceiver::new(rx));
        let bus = event_bus().unwrap();
        let watcher = DocsWatcher::new(bus.clone());
        watcher.run(mock_event_receiver);
        let sub = bus.subscriber();

        // when
        tx.send(DocsEvent::Other).unwrap();

        // then
        sub.try_recv(Duration::from_secs(2)).unwrap(); // should panic
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn errors_in_receiver_are_not_propagated_to_event_bus() {
        // given
        init_tracing();
        let erroneous_event_receiver = Box::new(ErroneousEventReceiver);
        let bus = event_bus().unwrap();
        let watcher = DocsWatcher::new(bus.clone());
        let sub = bus.subscriber();

        // when
        watcher.run(erroneous_event_receiver); // error ignored here

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
        fn recv(&self) -> std::result::Result<DocsEvent, EventReceiverErr> {
            Ok(self.rx.recv()?)
        }
    }

    struct ErroneousEventReceiver;

    impl EventReceiver for ErroneousEventReceiver {
        fn recv(&self) -> std::result::Result<DocsEvent, EventReceiverErr> {
            Err(EventReceiverErr::ReceiveError(RecvError))
        }
    }
}
