use crate::result::Result;
use crate::use_cases::event::{Event, Input, Output};

use std::sync::mpsc::{channel, Receiver, Sender};

pub fn channel_pipe() -> (Box<dyn Input>, Box<dyn Output>) {
    let (tx, rx) = channel();
    let input = Box::new(ChannelInput { rx });
    let output = Box::new(ChannelOutput { tx });
    (input, output)
}

#[derive(Debug)]
pub struct ChannelInput {
    rx: Receiver<Event>,
}

impl Input for ChannelInput {
    fn recv(&self) -> Result<Event> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct ChannelOutput {
    tx: Sender<Event>,
}

impl Output for ChannelOutput {
    fn send(&self, event: Event) -> Result<()> {
        unimplemented!()
    }
}
