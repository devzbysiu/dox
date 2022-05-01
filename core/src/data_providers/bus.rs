use crate::result::Result;
use crate::use_cases::bus::Bus;
use crate::use_cases::bus::{Event, Publisher, Subscriber};

pub struct LocalBus {
    eventador: eventador::Eventador,
}

impl LocalBus {
    pub fn new() -> Result<Self> {
        Ok(Self {
            eventador: eventador::Eventador::new(1024)?,
        })
    }
}

impl Bus for LocalBus {
    fn subscriber(&self) -> Box<dyn Subscriber> {
        Box::new(LocalSubscriber::new(self.eventador.subscribe()))
    }

    fn publisher(&self) -> Box<dyn Publisher> {
        Box::new(LocalPublisher::new(self.eventador.publisher()))
    }

    fn publish(&self, event: Event) -> Result<()> {
        self.eventador.publish(event);
        Ok(())
    }
}

pub struct LocalSubscriber {
    sub: eventador::Subscriber<Event>,
}

impl LocalSubscriber {
    fn new(sub: eventador::Subscriber<Event>) -> Self {
        Self { sub }
    }
}

impl Subscriber for LocalSubscriber {
    fn recv(&self) -> Result<Event> {
        Ok(self.sub.recv().to_owned())
    }
}

pub struct LocalPublisher {
    publ: eventador::Publisher,
}

impl LocalPublisher {
    fn new(publ: eventador::Publisher) -> Self {
        Self { publ }
    }
}

impl Publisher for LocalPublisher {
    fn publish(&mut self, event: Event) -> Result<()> {
        self.publ.send(event);
        Ok(())
    }
}
