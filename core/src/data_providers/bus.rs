use crate::result::Result;
use crate::use_cases::bus::Bus;
use crate::use_cases::bus::{Event, Publisher, Subscriber};

pub struct LocalBus;

impl Bus for LocalBus {
    fn subscriber(&self) -> Box<dyn Subscriber> {
        unimplemented!()
    }

    fn publisher(&self) -> Box<dyn Publisher> {
        unimplemented!()
    }

    fn publish(&self, event: &Event) -> Result<()> {
        unimplemented!()
    }
}

pub struct LocalSubscriber;

impl Subscriber for LocalSubscriber {
    fn recv(&self) -> Result<Event> {
        unimplemented!()
    }
}

pub struct LocalPublisher;

impl Publisher for LocalPublisher {
    fn publish(&self, event: &Event) -> Result<()> {
        unimplemented!()
    }
}
