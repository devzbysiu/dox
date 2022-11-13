use crate::entities::location::Location;
use crate::result::WatcherErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::receiver::{DocsEvent, EventRecv};

use std::thread;
use tracing::{debug, trace};

type Result<T> = std::result::Result<T, WatcherErr>;

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
        thread::spawn(move || -> Result<()> {
            debug!("watching thread spawned");
            let mut publ = self.bus.publisher();
            loop {
                trace!("waiting for event from watcher");
                match receiver.recv() {
                    Ok(DocsEvent::Created(path)) => {
                        debug!("got create file event on path: '{:?}'", path);
                        publ.send(BusEvent::NewDocs(Location::FS(vec![path])))?;
                    }
                    Ok(e) => trace!("event not supported in Watcher: '{}'", e),
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
    use crate::result::EventReceiverErr;
    use crate::testingtools::unit::create_test_shim;
    use crate::use_cases::bus::BusEvent;
    use crate::use_cases::receiver::EventReceiver;

    use anyhow::Result;
    use std::sync::mpsc::{Receiver, RecvError};

    #[test]
    fn created_docs_event_puts_new_docs_event_on_bus() -> Result<()> {
        // given
        init_tracing();
        let mut shim = create_test_shim()?;
        let mock_event_receiver = MockEventReceiver::new(shim.rx());
        DocsWatcher::new(shim.bus()).run(mock_event_receiver);

        // when
        shim.trigger_watcher()?;

        // then
        assert!(shim.event_on_bus(&BusEvent::NewDocs(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn other_docs_event_is_ignored() -> Result<()> {
        // given
        init_tracing();
        let mut shim = create_test_shim()?;
        let mock_event_receiver = MockEventReceiver::new(shim.rx());
        DocsWatcher::new(shim.bus()).run(mock_event_receiver);

        // when
        shim.mk_docs_event(DocsEvent::Other)?;

        // then
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn errors_in_receiver_are_not_propagated_to_event_bus() -> Result<()> {
        // given
        init_tracing();
        let erroneous_event_receiver = ErroneousEventReceiver::new();
        let shim = create_test_shim()?;
        let watcher = DocsWatcher::new(shim.bus());

        // when
        watcher.run(erroneous_event_receiver); // error ignored here

        // then
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    struct MockEventReceiver {
        rx: Receiver<DocsEvent>,
    }

    impl MockEventReceiver {
        fn new(rx: Receiver<DocsEvent>) -> Box<Self> {
            Box::new(Self { rx })
        }
    }

    impl EventReceiver for MockEventReceiver {
        fn recv(&self) -> std::result::Result<DocsEvent, EventReceiverErr> {
            Ok(self.rx.recv()?)
        }
    }

    struct ErroneousEventReceiver;

    impl ErroneousEventReceiver {
        fn new() -> Box<Self> {
            Box::new(Self)
        }
    }

    impl EventReceiver for ErroneousEventReceiver {
        fn recv(&self) -> std::result::Result<DocsEvent, EventReceiverErr> {
            Err(EventReceiverErr::ReceiveError(RecvError))
        }
    }
}
