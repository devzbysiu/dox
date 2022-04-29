use crate::result::Result;
use crate::use_cases::pipe::{ExternalEvent, Input, Output};

use std::sync::mpsc::{channel, Receiver, Sender};

/// Creates a local pipe which is connected via [`channel`].
///
/// Local means that no Inter Process Communication is made in this case.
#[allow(clippy::module_name_repetitions)]
pub fn channel_pipe() -> (Box<dyn Input>, Box<dyn Output>) {
    let (tx, rx) = channel();
    let input = Box::new(ChannelInput { rx });
    let output = Box::new(ChannelOutput { tx });
    (input, output)
}

/// Represents [`Input`] which allows receiving [`Event`]s.
#[derive(Debug)]
pub struct ChannelInput {
    rx: Receiver<ExternalEvent>,
}

impl Input for ChannelInput {
    fn recv(&self) -> Result<ExternalEvent> {
        Ok(self.rx.recv()?)
    }
}

/// Represents [`Output`] which allows sending [`Event`]s.
#[derive(Debug)]
pub struct ChannelOutput {
    tx: Sender<ExternalEvent>,
}

impl Output for ChannelOutput {
    fn send(&self, event: ExternalEvent) -> Result<()> {
        self.tx.send(event)?;
        Ok(())
    }
}
