use crate::entities::location::{Location, SafePathBuf};
use crate::result::EncrypterErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::cipher::CipherWrite;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub struct Encrypter {
    bus: EventBus,
}

impl Encrypter {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, cipher))]
    pub fn run(self, cipher: CipherWrite) {
        let sub = self.bus.subscriber();
        // TODO: improve tracing of threads somehow. Currently, it's hard to debug because threads
        // do not appear as separate tracing's scopes
        thread::spawn(move || -> Result<(), EncrypterErr> {
            let mut publ = self.bus.publisher();
            loop {
                match sub.recv()? {
                    BusEvent::EncryptionRequest(location) => {
                        debug!("encryption request: '{:?}', starting encryption", location);
                        let Location::FS(paths) = location;
                        let all_worked_out = paths
                            .par_iter()
                            .map(|path| encrypt(&cipher, path))
                            .inspect(report_errors)
                            .all(|r| r.is_ok());
                        if all_worked_out {
                            debug!("encryption finished");
                            publ.send(BusEvent::PipelineFinished)?;
                        }
                    }
                    e => debug!("event not supported in encrypter: '{}'", e),
                }
            }
        });
    }
}

fn encrypt(cipher: &CipherWrite, path: &SafePathBuf) -> Result<(), EncrypterErr> {
    let encrypted = cipher.encrypt(&fs::read(path)?)?;
    fs::write(path, encrypted)?;
    Ok(())
}

fn report_errors(res: &Result<(), EncrypterErr>) {
    if let Err(e) = res {
        error!("failed to encrypt: {:?}", e);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::factories::event_bus;
    use crate::configuration::telemetry::init_tracing;
    use crate::entities::user::FAKE_USER_EMAIL;
    use crate::result::CipherErr;
    use crate::testingtools::{mk_file, Spy, SubscriberExt};
    use crate::use_cases::bus::BusEvent;
    use crate::use_cases::cipher::CipherWriteStrategy;

    use anyhow::Result;
    use claim::assert_err;
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn cipher_is_used_when_encryption_request_appears() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spy, cipher_writer) = CipherSpy::working();
        let new_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
        let bus = event_bus()?;
        let encrypter = Encrypter::new(bus.clone());
        encrypter.run(cipher_writer);
        let mut publ = bus.publisher();

        // when
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![
            new_file.path,
        ])))?;

        // then
        assert!(cipher_spy.method_called());

        Ok(())
    }

    #[test]
    fn pipeline_finished_message_appears_after_encryption() -> Result<()> {
        // given
        init_tracing();
        let noop_cipher = Arc::new(NoOpCipher);
        let new_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
        let bus = event_bus()?;
        let encrypter = Encrypter::new(bus.clone());
        encrypter.run(noop_cipher);

        let mut publ = bus.publisher();
        let sub = bus.subscriber();

        // when
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![
            new_file.path,
        ])))?;

        let _event = sub.recv()?; // ignore EncryptionRequest message sent earliner

        // then
        assert_eq!(sub.recv()?, BusEvent::PipelineFinished);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_encrypter_fails() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_cipher) = CipherSpy::failing();
        let new_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;
        let bus = event_bus()?;
        let encrypter = Encrypter::new(bus.clone());
        encrypter.run(failing_cipher);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor
        let mut publ = bus.publisher();
        let sub = bus.subscriber();

        // when
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![
            new_file.path,
        ])))?;

        // then
        let _event = sub.recv()?; // ignore NewDocs event
        assert!(spy.method_called());
        assert_err!(sub.try_recv(Duration::from_secs(2))); // no more events on the bus

        Ok(())
    }

    #[test]
    #[should_panic(expected = "timed out waiting on channel")]
    fn encrypter_ignores_other_bus_events() {
        // given
        init_tracing();
        let noop_cipher = Arc::new(NoOpCipher);
        let _new_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into()).unwrap();
        let bus = event_bus().unwrap();
        let location = Location::FS(Vec::new());
        let ignored_events = [
            BusEvent::NewDocs(location.clone()),
            BusEvent::DataExtracted(Vec::new()),
            BusEvent::ThumbnailMade(location),
            BusEvent::Indexed(Vec::new()),
        ];
        let encrypter = Encrypter::new(bus.clone());
        encrypter.run(noop_cipher);

        let mut publ = bus.publisher();
        let sub = bus.subscriber();

        // when
        for event in &ignored_events {
            publ.send(event.clone()).unwrap();
        }

        // then
        // all events are still on the bus, no PipelineFinished emitted
        for _ in ignored_events {
            assert_ne!(sub.recv().unwrap(), BusEvent::PipelineFinished);
        }
        sub.try_recv(Duration::from_secs(2)).unwrap(); // should panic
    }

    #[test]
    fn failure_during_encryption_do_not_kill_service() -> Result<()> {
        // given
        let (spy, failing_repo_write) = CipherSpy::failing();
        let bus = event_bus()?;
        let new_file = mk_file(base64::encode(FAKE_USER_EMAIL), "some-file.jpg".into())?;

        let encrypter = Encrypter::new(bus.clone());
        encrypter.run(failing_repo_write);
        let mut publ = bus.publisher();
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![new_file
            .path
            .clone()])))?;
        assert!(spy.method_called());

        // when
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![
            new_file.path,
        ])))?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    struct CipherSpy;

    impl CipherSpy {
        fn working() -> (Spy, CipherWrite) {
            let (tx, rx) = channel();
            (Spy::new(rx), WorkingCipher::new(tx))
        }

        fn failing() -> (Spy, CipherWrite) {
            let (tx, rx) = channel();
            (Spy::new(rx), FailingCipher::new(tx))
        }
    }

    struct WorkingCipher {
        tx: Mutex<Sender<()>>,
    }

    impl WorkingCipher {
        fn new(tx: Sender<()>) -> Arc<Self> {
            Arc::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl CipherWriteStrategy for WorkingCipher {
        fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(Vec::new())
        }
    }

    struct FailingCipher {
        tx: Mutex<Sender<()>>,
    }

    impl FailingCipher {
        fn new(tx: Sender<()>) -> Arc<Self> {
            Arc::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl CipherWriteStrategy for FailingCipher {
        fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Err(CipherErr::ChachaError(chacha20poly1305::Error))
        }
    }

    struct NoOpCipher;

    impl CipherWriteStrategy for NoOpCipher {
        fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
            // nothing to do
            Ok(Vec::new())
        }
    }
}
