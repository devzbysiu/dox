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

    use crate::configuration::telemetry::init_tracing;
    use crate::result::CipherErr;
    use crate::testingtools::{create_test_shim, Spy};
    use crate::use_cases::bus::BusEvent;
    use crate::use_cases::cipher::CipherWriteStrategy;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Arc, Mutex};

    #[test]
    fn cipher_is_used_when_encryption_request_appears() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spy, cipher_writer) = CipherSpy::working();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher_writer);

        // when
        shim.trigger_encrypter()?;

        // then
        assert!(cipher_spy.method_called());

        Ok(())
    }

    #[test]
    fn pipeline_finished_message_appears_after_encryption() -> Result<()> {
        // given
        init_tracing();
        let noop_cipher = NoOpCipher::new();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(noop_cipher);

        // when
        shim.trigger_encrypter()?;

        shim.ignore_event()?; // ignore EncryptionRequest message sent earliner

        // then
        assert!(shim.pipeline_finished()?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_encrypter_fails() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_cipher) = CipherSpy::failing();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(failing_cipher);

        // when
        shim.trigger_encrypter()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(spy.method_called());
        assert!(shim.no_events_on_bus()); // no more events on the bus

        Ok(())
    }

    #[test]
    fn encrypter_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let noop_cipher = NoOpCipher::new();
        let mut shim = create_test_shim()?;
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::DataExtracted(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
        ];
        Encrypter::new(shim.bus()).run(noop_cipher);

        // when
        shim.send_events(&ignored_events)?;

        // then
        // all events are still on the bus, no PipelineFinished emitted
        assert!(shim.no_such_events(&[BusEvent::PipelineFinished], ignored_events.len())?);
        assert!(shim.no_events_on_bus()); // no more events on the bus

        Ok(())
    }

    #[test]
    fn failure_during_encryption_do_not_kill_service() -> Result<()> {
        // given
        let (spy, failing_repo_write) = CipherSpy::failing();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(failing_repo_write);
        shim.trigger_encrypter()?;
        assert!(spy.method_called());

        // when
        shim.trigger_encrypter()?;

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

    impl NoOpCipher {
        fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl CipherWriteStrategy for NoOpCipher {
        fn encrypt(&self, _src_buf: &[u8]) -> std::result::Result<Vec<u8>, CipherErr> {
            // nothing to do
            Ok(Vec::new())
        }
    }
}
