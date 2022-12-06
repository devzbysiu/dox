use crate::entities::location::{Location, SafePathBuf};
use crate::result::EncrypterErr;
use crate::use_cases::bus::{BusEvent, EventBus};
use crate::use_cases::cipher::CipherWrite;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::thread;
use tracing::{debug, error, instrument, trace, warn};

type Result<T> = std::result::Result<T, EncrypterErr>;

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
        thread::spawn(move || -> Result<()> {
            let mut publ = self.bus.publisher();
            loop {
                let ev = sub.recv()?;
                match ev.clone() {
                    BusEvent::EncryptDocument(location) | BusEvent::EncryptThumbnail(location) => {
                        if encrypt_all(&location, &cipher).is_ok() {
                            debug!("encryption finished");
                            publ.send(BusEvent::PipelineFinished)?;
                            continue;
                        }
                        error!("encryption failed");
                        publ.send(pick_response(&ev, location))?;
                    }
                    e => trace!("event not supported in encrypter: '{:?}'", e),
                }
            }
        });
    }
}

fn encrypt_all(location: &Location, cipher: &CipherWrite) -> Result<bool> {
    debug!("encryption request: '{:?}', starting encryption", location);
    let Location::FS(paths) = location;
    paths
        .par_iter()
        .map(|path| encrypt(cipher, path))
        .inspect(report_errors)
        .all(|r| r.is_ok())
        .then_some(true)
        .ok_or(EncrypterErr::AllOrNothing)
}

fn report_errors(res: &Result<()>) {
    if let Err(e) = res {
        error!("failed to encrypt: {:?}", e);
    }
}

fn encrypt(cipher: &CipherWrite, path: &SafePathBuf) -> Result<()> {
    let encrypted = cipher.encrypt(&fs::read(path)?)?;
    fs::write(path, encrypted)?;
    Ok(())
}

fn pick_response(ev: &BusEvent, location: Location) -> BusEvent {
    if matches!(ev, BusEvent::EncryptDocument(_)) {
        BusEvent::DocumentEncryptionFailed(location)
    } else {
        BusEvent::ThumbnailEncryptionFailed(location)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::services::encrypter::{failing, noop, tracked, working};
    use crate::testingtools::unit::create_test_shim;
    use crate::use_cases::bus::BusEvent;

    use anyhow::Result;
    use fake::{Fake, Faker};

    #[test]
    fn cipher_is_used_when_encrypt_thumbnail_event_appears() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spies, cipher) = tracked(&working());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_thumbail_encryption()?;

        // then
        assert!(cipher_spies.encrypt_called());

        Ok(())
    }

    #[test]
    fn cipher_is_used_when_encrypt_document_event_appears() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spies, cipher) = tracked(&working());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_document_encryption()?;

        // then
        assert!(cipher_spies.encrypt_called());

        Ok(())
    }

    #[test]
    fn pipeline_finished_message_appears_after_thumbnail_encryption() -> Result<()> {
        // given
        init_tracing();
        let cipher = noop();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_thumbail_encryption()?;

        shim.ignore_event()?; // ignore encryption message sent earliner

        // then
        assert!(shim.pipeline_finished()?);

        Ok(())
    }

    #[test]
    fn pipeline_finished_message_appears_after_document_encryption() -> Result<()> {
        // given
        init_tracing();
        let cipher = noop();
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_document_encryption()?;

        shim.ignore_event()?; // ignore encryption message sent earliner

        // then
        assert!(shim.pipeline_finished()?);

        Ok(())
    }

    #[test]
    fn thumbnail_encryption_failed_event_appears_when_thumbnail_encryption_fails() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spies, cipher) = tracked(&failing());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_thumbail_encryption()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(cipher_spies.encrypt_called());
        assert!(shim.event_on_bus(&BusEvent::ThumbnailEncryptionFailed(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn document_encryption_failed_event_appears_when_document_encryption_fails() -> Result<()> {
        // given
        init_tracing();
        let (cipher_spies, cipher) = tracked(&failing());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.trigger_document_encryption()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(cipher_spies.encrypt_called());
        assert!(shim.event_on_bus(&BusEvent::DocumentEncryptionFailed(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn encrypter_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let cipher = noop();
        let mut shim = create_test_shim()?;
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::DataExtracted(Faker.fake()),
            BusEvent::DocsMoved(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::ThumbnailRemoved,
            BusEvent::DataRemoved,
        ];
        Encrypter::new(shim.bus()).run(cipher.write());

        // when
        shim.send_events(&ignored_events)?;

        // then
        // all events are still on the bus, no PipelineFinished, ThumbnailEncryptionFailed emited
        // and DocumentEncryptionFailed
        for _ in 0..ignored_events.len() {
            let received = shim.recv_event()?;
            assert!(!matches!(received, BusEvent::PipelineFinished));
            assert!(!matches!(received, BusEvent::ThumbnailEncryptionFailed(_)));
            assert!(!matches!(received, BusEvent::DocumentEncryptionFailed(_)));
        }
        assert!(shim.no_events_on_bus()); // no more events on the bus

        Ok(())
    }

    #[test]
    fn failure_during_thumbnail_encryption_do_not_kill_service() -> Result<()> {
        // given
        let (cipher_spies, cipher) = tracked(&failing());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());
        shim.trigger_thumbail_encryption()?;
        assert!(cipher_spies.encrypt_called());

        // when
        shim.trigger_thumbail_encryption()?;

        // then
        assert!(cipher_spies.encrypt_called());

        Ok(())
    }

    #[test]
    fn failure_during_document_encryption_do_not_kill_service() -> Result<()> {
        // given
        let (cipher_spies, cipher) = tracked(&failing());
        let mut shim = create_test_shim()?;
        Encrypter::new(shim.bus()).run(cipher.write());
        shim.trigger_document_encryption()?;
        assert!(cipher_spies.encrypt_called());

        // when
        shim.trigger_document_encryption()?;

        // then
        assert!(cipher_spies.encrypt_called());

        Ok(())
    }
}
