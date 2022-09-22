//! This module is a concrete implementation of the interfaces defined in [`crate::use_cases::bus`].
//!
//! It uses [`eventador`] library to build local event bus implementation.
use std::fmt::Debug;

use crate::result::Result;
use crate::use_cases::bus::{
    Bus, BusEvent, EventBus, EventPublisher, EventSubscriber, Publisher, Subscriber,
};

const BUS_CAPACITY: u64 = 1024; // TODO: take care of this `capacity`

/// Event bus for communicating between core components inside the same process space.
///
/// It's using [`Eventador`] to handle message queue and message delivery.
pub struct LocalBus {
    eventador: eventador::Eventador,
}

impl Debug for LocalBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("local bus")
    }
}

impl LocalBus {
    pub fn new() -> Result<Self> {
        Ok(Self {
            eventador: eventador::Eventador::new(BUS_CAPACITY)?,
        })
    }
}

impl Bus for LocalBus {
    fn subscriber(&self) -> EventSubscriber {
        Box::new(LocalSubscriber::new(self.eventador.subscribe()))
    }

    fn publisher(&self) -> EventPublisher {
        Box::new(LocalPublisher::new(self.eventador.publisher()))
    }

    fn share(&self) -> EventBus {
        Box::new(Self {
            eventador: self.eventador.clone(),
        })
    }
}

/// Represents Subscriber of [`LocalBus`].
///
/// It allows to receive [`Event`]s.
pub struct LocalSubscriber {
    sub: eventador::Subscriber<BusEvent>,
}

impl LocalSubscriber {
    fn new(sub: eventador::Subscriber<BusEvent>) -> Self {
        Self { sub }
    }
}

impl Subscriber for LocalSubscriber {
    fn recv(&self) -> Result<BusEvent> {
        Ok(self.sub.recv().to_owned())
    }
}

/// Represents Publisher of [`LocalBus`].
///
/// It allows to send [`Event`]s.
pub struct LocalPublisher {
    publ: eventador::Publisher,
}

impl LocalPublisher {
    fn new(publ: eventador::Publisher) -> Self {
        Self { publ }
    }
}

impl Publisher for LocalPublisher {
    fn send(&mut self, event: BusEvent) -> Result<()> {
        self.publ.send(event);
        Ok(())
    }
}
