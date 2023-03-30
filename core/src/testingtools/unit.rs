use crate::configuration::factories::event_bus;
use crate::entities::document::DocDetails;
use crate::entities::location::{Location, SafePathBuf};
use crate::entities::user::FAKE_USER_EMAIL;
use crate::testingtools::TestConfig;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher, EventSubscriber};
use crate::use_cases::receiver::DocsEvent;

use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use std::fs::{self, create_dir_all};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
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
    let (tx, rx) = channel();
    let rx = Some(rx);
    let test_file = mk_file(b64.encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
    let bus = event_bus()?;
    let publ = bus.publisher();
    let sub = bus.subscriber();
    let config = TestConfig::new()?;
    Ok(TestShim {
        rx,
        tx,
        test_file,
        bus,
        publ,
        sub,
        config,
    })
}

pub struct TestShim {
    rx: Option<Receiver<DocsEvent>>,
    tx: Sender<DocsEvent>,
    test_file: TestFile,
    bus: EventBus,
    publ: EventPublisher,
    sub: EventSubscriber,
    config: TestConfig,
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

    pub fn config(&self) -> &TestConfig {
        &self.config
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

    pub fn recv_event(&self) -> Result<BusEvent> {
        Ok(self.sub.recv()?)
    }

    pub fn trigger_extractor(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::DocsMoved(test_location))?;
        Ok(())
    }

    pub fn trigger_mover(&mut self) -> Result<()> {
        let test_location = self.test_file.location.clone();
        self.publ.send(BusEvent::NewDocs(test_location))?;
        Ok(())
    }

    pub fn event_on_bus(&self, event: &BusEvent) -> Result<bool> {
        // let received = self.sub.recv()?;
        // debug!("received event: {:?}", received);
        // Ok(*event == received)

        let (tx, rx) = channel();
        let sub = self.sub.clone();
        let t = thread::spawn(move || -> Result<()> {
            tx.send(sub.recv()?)?;
            Ok(())
        });

        thread::sleep(Duration::from_secs(2));

        match rx.try_recv() {
            Ok(received) => Ok(*event == received),
            Err(TryRecvError::Empty) => {
                drop(rx);
                drop(t);
                // receiving event took more than 2 seconds
                Ok(false)
            }
            Err(TryRecvError::Disconnected) => unreachable!(),
        }
    }

    pub fn test_location(&self) -> Location {
        self.test_file.location.clone()
    }

    pub fn dst_doc_location(&self) -> Location {
        let docs_dir = self.config.docs_dir.path();
        let src_path = &self.test_file.path;
        let filename = src_path.filename();
        Location::FS(vec![docs_dir.join(filename).into()])
    }

    pub fn trigger_indexer(&mut self, details: Vec<DocDetails>) -> Result<()> {
        self.publ.send(BusEvent::DataExtracted(details))?;
        Ok(())
    }

    pub fn trigger_thumbnailer(&mut self) -> Result<()> {
        self.publ.send(BusEvent::DocsMoved(self.test_location()))?;
        Ok(())
    }

    pub fn trigger_watcher(&self) -> Result<()> {
        let file_path = self.test_file.path.clone();
        self.tx.send(DocsEvent::Created(file_path))?;
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

    pub fn rx(&mut self) -> Receiver<DocsEvent> {
        self.rx.take().unwrap()
    }

    pub fn mk_docs_event(&self, event: DocsEvent) -> Result<()> {
        self.tx.send(event)?;
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
