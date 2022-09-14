use crate::result::Result;

use std::path::PathBuf;

pub type Watcher = Box<dyn EventWatcher>;

pub trait EventWatcher: Send {
    fn recv(&self) -> Result<WatcherEvent>;
}

#[derive(Debug)]
pub enum WatcherEvent {
    Created(PathBuf),
    Other,
}
