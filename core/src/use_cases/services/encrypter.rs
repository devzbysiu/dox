use crate::entities::location::Location;
use crate::result::Result;
use crate::use_cases::bus::{Bus, BusEvent};
use crate::use_cases::cipher::CipherWrite;

use std::fs;
use std::thread;
use tracing::{debug, instrument, warn};

pub struct Encrypter<'a> {
    bus: &'a dyn Bus,
}

impl<'a> Encrypter<'a> {
    pub fn new(bus: &'a dyn Bus) -> Self {
        Self { bus }
    }

    #[instrument(skip(self, cipher))]
    pub fn run(&self, cipher: CipherWrite) {
        let sub = self.bus.subscriber();
        let mut publ = self.bus.publisher();
        // TODO: improve tracing of threads somehow. Currently, it's hard to debug because threads
        // do not appear as separate tracing's scopes
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    BusEvent::EncryptionRequest(location) => {
                        debug!("encryption request: '{:?}', starting encryption", location);
                        let Location::FS(paths) = location;
                        for path in paths {
                            let encrypted = cipher.encrypt(&fs::read(&path)?)?;
                            fs::write(path, encrypted)?;
                        }
                        debug!("encryption finished");
                        publ.send(BusEvent::PipelineFinished)?;
                    }
                    e => debug!("event not supported in encrypter: {}", e.to_string()),
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::data_providers::bus::LocalBus;
    use crate::result::DoxErr;
    use crate::use_cases::cipher::CipherWriteStrategy;

    use anyhow::Result;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::Mutex;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn cipher_is_used_when_encryption_request_appears() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spy, cipher_writer) = CipherSpy::new();
        let tmp_dir = tempdir()?;
        let tmp_file_path = tmp_dir.path().join("tmp_file");
        fs::write(&tmp_file_path, "anything")?;

        // when
        let encrypter = Encrypter::new(&bus);
        encrypter.run(cipher_writer);

        let bus = LocalBus::new()?;
        let mut publ = bus.publisher();
        let sub = bus.subscriber();
        publ.send(BusEvent::EncryptionRequest(Location::FS(vec![
            tmp_file_path,
        ])))?;

        let _ = sub.recv()?; // ignore EncryptionRequest message sent earliner

        // then
        assert!(cipher_spy.cipher_has_been_called());
        assert_eq!(sub.recv()?, BusEvent::PipelineFinished);

        Ok(())
    }

    struct CipherSpy {
        tx: Mutex<Sender<()>>,
    }

    impl CipherSpy {
        fn new() -> (Spy, CipherWrite) {
            let (tx, rx) = channel();
            (Spy::new(rx), Box::new(Self { tx: Mutex::new(tx) }))
        }
    }

    impl CipherWriteStrategy for CipherSpy {
        fn encrypt(&self, _src_buf: &[u8]) -> crate::result::Result<Vec<u8>> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .map_err(|_| DoxErr::Encryption(chacha20poly1305::Error))?;
            Ok(Vec::new())
        }
    }

    struct Spy {
        rx: Receiver<()>,
    }

    impl Spy {
        fn new(rx: Receiver<()>) -> Self {
            Self { rx }
        }

        fn cipher_has_been_called(&self) -> bool {
            self.rx.recv_timeout(Duration::from_secs(2)).is_ok()
        }
    }
}