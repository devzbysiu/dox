//! Abstraction for generating thumbnail of received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::ThumbnailerErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::config::Config;
use crate::use_cases::fs::Fs;

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::path::{Path, PathBuf};
use std::thread;
use tracing::{debug, error, instrument, trace, warn};

pub type ThumbnailerCreator = Box<dyn ThumbnailerFactory>;
pub type Thumbnailer = Box<dyn ThumbnailMaker>;
type Result<T> = std::result::Result<T, ThumbnailerErr>;

pub struct ThumbnailGenerator {
    cfg: Config,
    bus: EventBus,
    tp: ThreadPool,
}

impl ThumbnailGenerator {
    pub fn new<C: Into<Config>>(cfg: C, bus: EventBus) -> Result<Self> {
        let cfg = cfg.into();
        let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
        Ok(Self { cfg, bus, tp })
    }

    #[instrument(skip(self, factory, fs))]
    pub fn run(self, factory: ThumbnailerCreator, fs: Fs) {
        thread::spawn(move || -> Result<()> {
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    BusEvent::DocsMoved(loc) => self.do_thumbnail(loc, &factory)?,
                    BusEvent::ThumbnailEncryptionFailed(loc) => self.cleanup(loc, &fs),
                    e => trace!("event not supported in ThumbnailGenerator: '{:?}'", e),
                }
            }
        });
    }

    #[instrument(skip(self, factory))]
    fn do_thumbnail(&self, loc: Location, factory: &ThumbnailerCreator) -> Result<()> {
        debug!("NewDocs in: '{:?}', creating thumbnail", loc);
        let thumbnailer = factory.make(&loc.extension()?);
        let publ = self.bus.publisher();
        let dir = self.cfg.thumbnails_dir.clone();
        self.tp.spawn(move || {
            if let Err(e) = mk_thumbnail(&loc, &thumbnailer, &dir, publ) {
                error!("thumbnail generation failed: '{}'", e);
            }
        });
        Ok(())
    }

    #[instrument(skip(self, fs))]
    fn cleanup(&self, loc: Location, fs: &Fs) {
        debug!("pipeline failed, removing thumbnail");
        let fs = fs.clone();
        let publ = self.bus.publisher();
        self.tp.spawn(move || {
            if let Err(e) = remove_thumbnail(&loc, &fs, publ) {
                error!("thumbnail removal failed: '{}'", e);
            }
        });
    }
}

#[instrument(skip(prepr, publ))]
fn mk_thumbnail(
    loc: &Location,
    prepr: &Thumbnailer,
    dir: &PathBuf,
    publ: EventPublisher,
) -> Result<()> {
    let thumbnails_dir = dir.as_ref();
    let thumbnail_loc = prepr.mk_thumbnail(loc, thumbnails_dir)?;
    debug!("creating thumbnail finished");
    publ.send(BusEvent::ThumbnailMade(thumbnail_loc.clone()))?;
    debug!("sending encryption request for: '{:?}'", thumbnail_loc);
    publ.send(BusEvent::EncryptThumbnail(thumbnail_loc))?;
    Ok(())
}

#[instrument(skip(fs, publ))]
fn remove_thumbnail(loc: &Location, fs: &Fs, publ: EventPublisher) -> Result<()> {
    let Location::FS(paths) = loc;
    for path in paths {
        fs.rm_file(path)?;
        debug!("removed '{}'", path);
    }
    publ.send(BusEvent::ThumbnailRemoved)?;
    debug!("thumbnail removed");
    Ok(())
}

pub trait ThumbnailMaker: Send {
    fn mk_thumbnail(&self, location: &Location, thumbnails_dir: &Path) -> Result<Location>;
}

pub trait ThumbnailerFactory: Sync + Send {
    fn make(&self, ext: &Ext) -> Thumbnailer;
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::services::fs::{
        noop as noop_fs, tracked as tracked_fs, working as working_fs,
    };
    use crate::testingtools::services::thumbnailer::{
        factory, failing, noop as noop_thumbnailer, tracked, working,
    };
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::time::Duration;

    #[test]
    fn thumbnailer_is_used_to_generate_thumbnail() -> Result<()> {
        // given
        init_tracing();
        let (thumbnailer_spies, thumbnailer) = tracked(working());
        let factory_stub = factory(vec![thumbnailer]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start ThumbnailGenerator

        // when
        shim.trigger_thumbnailer()?;

        // then
        assert!(thumbnailer_spies.mk_thumbnail_called());

        Ok(())
    }

    #[test]
    fn thumbnail_made_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_thumbnailer()]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_thumbnailer()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(shim.event_on_bus(&BusEvent::ThumbnailMade(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn thumbnail_generator_emits_encrypt_thumbnail_event_on_success() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_thumbnailer()]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_thumbnailer()?;

        shim.ignore_event()?; // ignore NewDocs event
        shim.ignore_event()?; // ignore TextExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::EncryptThumbnail(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_thumbnailer_fails() -> Result<()> {
        // given
        init_tracing();
        let (thumbnailer_spies, thumbnailer) = tracked(failing());
        let factory_stub = factory(vec![thumbnailer]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start ThumbnailGenerator

        // when
        shim.trigger_thumbnailer()?;
        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(thumbnailer_spies.mk_thumbnail_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn thumbnailer_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_thumbnailer()]);
        let mut shim = create_test_shim()?;
        let ignored_events = [
            BusEvent::NewDocs(Faker.fake()),
            BusEvent::DataExtracted(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::EncryptDocument(Faker.fake()),
            BusEvent::DocumentEncryptionFailed(Faker.fake()),
            BusEvent::PipelineFinished,
        ];
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());

        // when
        shim.send_events(&ignored_events)?;

        // then
        // no ThumbnailMade, ThumbnailRemoved and EncryptThumbnail on the bus
        for _ in 0..ignored_events.len() {
            let received_event = shim.recv_event()?;
            assert!(!matches!(received_event, BusEvent::ThumbnailMade(_)));
            assert!(!matches!(received_event, BusEvent::EncryptThumbnail(_)));
            assert!(!matches!(received_event, BusEvent::ThumbnailRemoved));
        }
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_thumbnail_creation_do_not_kill_service() -> Result<()> {
        // given
        init_tracing();
        let (thumbnailer_spies1, thumbnailer1) = tracked(failing());
        let (thumbnailer_spies2, thumbnailer2) = tracked(failing());
        let factory_stub = factory(vec![thumbnailer1, thumbnailer2]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start ThumbnailGenerator

        shim.trigger_thumbnailer()?;
        assert!(thumbnailer_spies1.mk_thumbnail_called());

        // when
        shim.trigger_thumbnailer()?;

        // then
        assert!(thumbnailer_spies2.mk_thumbnail_called());

        Ok(())
    }

    #[test]
    fn when_thumbnail_encryption_failed_event_appears_filesystem_removes_thumbnail() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_thumbnailer()]);
        let (fs_spies, working_fs) = tracked_fs(working_fs());
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, working_fs);
        thread::sleep(Duration::from_secs(1)); // allow to start ThumbnailGenerator

        // when
        shim.trigger_thumbnail_encryption_failure()?;

        // then
        assert!(fs_spies.rm_file_called());

        Ok(())
    }
}
