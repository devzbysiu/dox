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

    fn publish(&self, event: Event) -> Result<()> {
        unimplemented!()
    }
}

pub struct LocalSubscriber {
    sub: eventador::Subscriber<Event>,
}

impl Subscriber for LocalSubscriber {
    fn recv(&self) -> Result<Event> {
        Ok(self.sub.recv().to_owned())
    }
}

pub struct LocalPublisher {
    publ: eventador::Publisher,
}

impl Publisher for LocalPublisher {
    fn publish(&self, event: Event) -> Result<()> {
        self.publ.send(event);
        Ok(())
    }
}
