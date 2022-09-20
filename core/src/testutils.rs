use crate::entities::location::SafePathBuf;
use crate::use_cases::bus::{BusEvent, EventSubscriber};

use anyhow::{anyhow, Result};
use std::fs;
use std::fs::create_dir_all;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, TempDir};

pub trait SubscriberExt {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent>;
}

impl SubscriberExt for EventSubscriber {
    fn try_recv(self, timeout: Duration) -> Result<BusEvent> {
        let (done_tx, done_rx) = channel();
        let handle = thread::spawn(move || -> Result<()> {
            let event = self.recv()?;
            done_tx.send(event)?;
            Ok(())
        });

        match done_rx.recv_timeout(timeout) {
            Ok(event) => {
                if let Err(e) = handle.join() {
                    panic!("failed to join thread: {:?}", e);
                }
                Ok(event)
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}

pub fn mk_file(user_dir_name: String, filename: String) -> Result<NewFile> {
    let tmp_dir = tempdir()?;
    let user_dir = tmp_dir.path().join(user_dir_name);
    create_dir_all(&user_dir)?;
    let path = user_dir.join(filename);
    fs::write(&path, "anything")?;
    let path = SafePathBuf::new(path);
    Ok(NewFile {
        _temp_dir: tmp_dir,
        path,
    })
}

pub struct NewFile {
    _temp_dir: TempDir,
    pub path: SafePathBuf,
}

pub struct Spy {
    rx: Receiver<()>,
}

impl Spy {
    pub fn new(rx: Receiver<()>) -> Self {
        Self { rx }
    }

    pub fn method_called(&self) -> bool {
        self.rx.recv_timeout(Duration::from_secs(2)).is_ok()
    }
}
