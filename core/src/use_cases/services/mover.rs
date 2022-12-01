//! Abstraction for moving received document to correct place.
use crate::entities::location::{Location, SafePathBuf};
use crate::helpers::PathRefExt;
use crate::result::MoverErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::config::Config;
use crate::use_cases::fs::Fs;

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::path::PathBuf;
use std::thread;
use tracing::{debug, error, instrument, trace, warn};

type Result<T> = std::result::Result<T, MoverErr>;

pub struct DocumentMover {
    cfg: Config,
    bus: EventBus,
    tp: ThreadPool,
}

impl DocumentMover {
    pub fn new<C: Into<Config>>(cfg: C, bus: EventBus) -> Result<Self> {
        let cfg = cfg.into();
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        Ok(Self { cfg, bus, tp })
    }

    #[instrument(skip(self, fs))]
    pub fn run(self, fs: Fs) {
        thread::spawn(move || -> Result<()> {
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    BusEvent::NewDocs(loc) => self.move_doc(loc, &fs),
                    BusEvent::DocumentEncryptionFailed(loc) => self.cleanup(loc, &fs),
                    e => trace!("event not supported in DocumentMover: '{:?}'", e),
                }
            }
        });
    }

    #[instrument(skip(self, fs))]
    fn move_doc(&self, loc: Location, fs: &Fs) {
        debug!("NewDocs in: '{:?}', moving to correct location", loc);
        let publ = self.bus.publisher();
        let dir = self.cfg.docs_dir.clone();
        let fs = fs.clone();
        self.tp.spawn(move || {
            if let Err(e) = move_document(&loc, &fs, &dir, publ) {
                error!("failed to move doc: '{}'", e);
            }
        });
    }

    #[instrument(skip(self, fs))]
    fn cleanup(&self, loc: Location, fs: &Fs) {
        debug!("pipeline failed, removing document");
        let fs = fs.clone();
        self.tp.spawn(move || {
            if let Err(e) = remove_document(&loc, &fs) {
                error!("document removal failed: '{}'", e);
            }
        });
    }
}

#[instrument(skip(fs, publ))]
fn move_document(loc: &Location, fs: &Fs, dir: &PathBuf, mut publ: EventPublisher) -> Result<()> {
    let Location::FS(paths) = loc;
    let mut dst_paths = Vec::new();
    for path in paths {
        let dst_path = dir.join(path.filename());
        fs.mv_file(path, &dst_path)?;
        dst_paths.push(SafePathBuf::new(dst_path));
    }
    debug!("moving finished");
    publ.send(BusEvent::DocsMoved(Location::FS(dst_paths)))?;
    Ok(())
}

#[instrument(skip(fs))]
fn remove_document(loc: &Location, fs: &Fs) -> Result<()> {
    let Location::FS(paths) = loc;
    for path in paths {
        fs.rm_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::services::fs::{failing_fs, noop_fs, tracked_fs};
    use crate::testingtools::unit::create_test_shim;
    use crate::testingtools::TestConfig;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::time::Duration;

    #[test]
    fn fs_is_used_to_move_document() -> Result<()> {
        // given
        init_tracing();
        let (fs_spies, fs) = tracked_fs(noop_fs());
        let mut shim = create_test_shim()?;
        DocumentMover::new(TestConfig::new()?, shim.bus())?.run(fs);
        thread::sleep(Duration::from_secs(1)); // allow to start DocumentMover

        // when
        shim.trigger_mover()?;

        // then
        assert!(fs_spies.mv_file_called());

        Ok(())
    }

    #[test]
    fn docs_moved_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let mut shim = create_test_shim()?;
        DocumentMover::new(shim.config(), shim.bus())?.run(noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start DocumentMover

        // when
        shim.trigger_mover()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(shim.event_on_bus(&BusEvent::DocsMoved(shim.dst_doc_location()))?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_mover_fails() -> Result<()> {
        // given
        init_tracing();
        let (fs_spies, fs) = tracked_fs(failing_fs());
        let mut shim = create_test_shim()?;
        DocumentMover::new(Config::default(), shim.bus())?.run(fs);
        thread::sleep(Duration::from_secs(1)); // allow to start DocumentMover

        // when
        shim.trigger_mover()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(fs_spies.mv_file_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn mover_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let mut shim = create_test_shim()?;
        // TODO: Update ignored events in the rest of the services
        let ignored_events = [
            BusEvent::DataExtracted(Faker.fake()),
            BusEvent::DocsMoved(Faker.fake()),
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::EncryptDocument(Faker.fake()),
            BusEvent::EncryptThumbnail(Faker.fake()),
            BusEvent::DocumentEncryptionFailed(Faker.fake()),
            BusEvent::ThumbnailEncryptionFailed(Faker.fake()),
            BusEvent::PipelineFinished,
        ];
        DocumentMover::new(Config::default(), shim.bus())?.run(noop_fs());

        // when
        shim.send_events(&ignored_events)?;

        // then
        // all events are still on the bus, no DocsMoved emitted
        shim.no_such_events(
            &[
                // TODO: this shouldn't use specific values - any DataExtracted and EncryptionRequest
                // event (with any data) should make this test fail
                BusEvent::DocsMoved(Faker.fake()),
            ],
            ignored_events.len(),
        )?;
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_moving_do_not_kill_service() -> Result<()> {
        // given
        init_tracing();
        let (fs_spies, fs) = tracked_fs(failing_fs());
        let mut shim = create_test_shim()?;
        DocumentMover::new(Config::default(), shim.bus())?.run(fs);
        thread::sleep(Duration::from_secs(1)); // allow to start DocumentMover

        shim.trigger_mover()?;
        assert!(fs_spies.mv_file_called());

        // when
        shim.trigger_mover()?;

        // then
        assert!(fs_spies.mv_file_called());

        Ok(())
    }

    #[test]
    fn when_document_encryption_failed_event_appears_filesystem_removes_document() -> Result<()> {
        // given
        init_tracing();
        let (fs_spies, fs) = tracked_fs(noop_fs());
        let mut shim = create_test_shim()?;
        DocumentMover::new(Config::default(), shim.bus())?.run(fs);
        thread::sleep(Duration::from_secs(1)); // allow to start DocumentMover

        // when
        shim.trigger_document_encryption_failure()?;

        // then
        assert!(fs_spies.rm_file_called());

        Ok(())
    }
}
