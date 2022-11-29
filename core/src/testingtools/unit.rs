#![allow(unused)] // TODO: remove this

use crate::configuration::factories::event_bus;
use crate::entities::document::DocDetails;
use crate::entities::location::{Location, SafePathBuf};
use crate::entities::user::FAKE_USER_EMAIL;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};
use crate::use_cases::receiver::DocsEvent;

use anyhow::{anyhow, Result};
use retry::{retry, OperationResult};
use std::fs::{self, create_dir_all};
use std::sync::mpsc::{channel, Receiver, Sender};
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

pub fn create_test_shim() -> Result<TestShim> {
    let test_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
    let bus = event_bus()?;
    let publ = bus.publisher();
    let sub = bus.subscriber();
    let (tx, rx) = channel();
    let rx = Some(rx);
    Ok(TestShim {
        rx,
        tx,
        test_file,
        bus,
        publ,
        sub,
    })
}

pub struct TestShim {
    rx: Option<Receiver<DocsEvent>>,
    tx: Sender<DocsEvent>,
    test_file: TestFile,
    bus: EventBus,
    publ: EventPublisher,
    sub: EventSubscriber,
}

impl TestShim {
    pub fn trigger_thumbail_encryption(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::EncryptThumbnail(test_location))?;
        Ok(())
    }

    pub fn trigger_document_encryption(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::EncryptDocument(test_location))?;
        Ok(())
    }

    pub fn bus(&self) -> EventBus {
        self.bus.clone()
    }

    pub fn ignore_event(&self) -> Result<()> {
        let _event = self.sub.recv()?; // ignore message sent earliner
        Ok(())
    }

    pub fn pipeline_finished(&self) -> Result<bool> {
        let event = self.sub.recv()?;
        Ok(event == BusEvent::PipelineFinished)
    }

    pub fn no_events_on_bus(self) -> bool {
        self.sub.try_recv(Duration::from_secs(2)).is_err()
    }

    pub fn send_events(&mut self, events: &[BusEvent]) -> Result<()> {
        for event in events {
            self.publ.send(event.clone())?;
        }
        Ok(())
    }

    pub fn no_such_events(&self, ignored: &[BusEvent], max_events: usize) -> Result<bool> {
        for i in 0..max_events {
            let received = self.sub.recv()?;
            for event in ignored {
                if *event == received {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    pub fn trigger_extractor(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::DocsMoved(test_location))?;
        Ok(())
    }

    // TODO: this should have timeout, otherwise it will hung the tests.
    // Currently, timeout is implemented in [`SubscriberExt`] but it spawns separate thread for
    // waiting for an event. Because of that, it consumes `self` and this cannot be done here
    // because I'm accepting `&self`.
    pub fn event_on_bus(&self, event: &BusEvent) -> Result<bool> {
        Ok(*event == self.sub.recv()?)
    }

    pub fn test_location(&self) -> Location {
        self.test_file.location.clone()
    }

    // TODO: this should take data the indexer should be triggered with - do that also for other
    // trigger_* methods
    pub fn trigger_indexer(&mut self, details: Vec<DocDetails>) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::DataExtracted(details))?;
        Ok(())
    }

    pub fn trigger_preprocessor(&mut self) -> Result<()> {
        self.publ.send(BusEvent::DocsMoved(self.test_location()))?;
        Ok(())
    }

    pub fn rx(&mut self) -> Receiver<DocsEvent> {
        self.rx.take().unwrap()
    }

    // TODO: I think it would be better to explicitly pass the event being sent - it's cleaner in
    // the test. This should be changed for all trigger_* methods
    pub fn trigger_watcher(&self) -> Result<()> {
        let file_path = self.test_file.path.clone();
        self.tx.send(DocsEvent::Created(file_path))?;
        Ok(())
    }

    pub fn mk_docs_event(&self, event: DocsEvent) -> Result<()> {
        self.tx.send(event)?;
        Ok(())
    }

    pub fn trigger_thumbnail_encryption_failure(&mut self) -> Result<()> {
        let loc = self.test_location();
        self.publ.send(BusEvent::ThumbnailEncryptionFailed(loc))?;
        Ok(())
    }

    pub fn trigger_document_encryption_failure(&mut self) -> Result<()> {
        let loc = self.test_location();
        self.publ.send(BusEvent::DocumentEncryptionFailed(loc))?;
        Ok(())
    }
}

fn mk_file(user_dir_name: String, filename: String) -> Result<TestFile> {
    let tmp_dir = tempdir()?;
    let user_dir = tmp_dir.path().join(user_dir_name);
    create_dir_all(&user_dir)?;
    let path = user_dir.join(filename);
    fs::write(&path, "anything")?;
    let path = SafePathBuf::new(path);
    Ok(TestFile {
        _temp_dir: tmp_dir,
        location: Location::FS(vec![path.clone()]),
        path,
    })
}

pub struct TestFile {
    _temp_dir: TempDir,
    pub path: SafePathBuf,
    pub location: Location,
}
