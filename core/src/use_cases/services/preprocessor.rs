//! Abstraction for generating thumbnail of received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::PreprocessorErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::config::Config;
use crate::use_cases::fs::Fs;

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::path::{Path, PathBuf};
use std::thread;
use tracing::{debug, error, instrument, trace, warn};

pub type PreprocessorCreator = Box<dyn PreprocessorFactory>;
pub type Preprocessor = Box<dyn FilePreprocessor>;
type Result<T> = std::result::Result<T, PreprocessorErr>;

/// Generates thumbnail either for PDF file or image file when [`Event::NewDocs`] appears on the
/// bus.
///
/// Depending on the [`Location::extension`], specific preprocessor is selected (see
/// [`FilePreprocessor`]). It then calls [`FilePreprocessor::preprocess`] method.
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
    pub fn run(self, factory: PreprocessorCreator, fs: Fs) {
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
    fn do_thumbnail(&self, loc: Location, factory: &PreprocessorCreator) -> Result<()> {
        debug!("NewDocs in: '{:?}', starting preprocessing", loc);
        let preprocessor = factory.make(&loc.extension()?);
        let publ = self.bus.publisher().clone();
        let dir = self.cfg.thumbnails_dir.clone();
        self.tp.spawn(move || {
            if let Err(e) = preprocess(&loc, &preprocessor, &dir, publ) {
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
fn preprocess(
    loc: &Location,
    prepr: &Preprocessor,
    dir: &PathBuf,
    publ: EventPublisher,
) -> Result<()> {
    let thumbnails_dir = dir.as_ref();
    let thumbnail_loc = prepr.preprocess(loc, thumbnails_dir)?;
    debug!("preprocessing finished");
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

/// Abstracts the process of preprocessing received document.
///
/// This happens right after the document was received. See
/// [`Indexer::run`](crate::use_cases::indexer::Indexer::run).
pub trait FilePreprocessor: Send {
    /// Take source location as the input and the parent directory for the output.
    /// Returns the final location of the preprocessing.
    fn preprocess(&self, location: &Location, thumbnails_dir: &Path) -> Result<Location>;
}

/// Creates [`Preprocessor`].
pub trait PreprocessorFactory: Sync + Send {
    /// Creates [`Preprocessor`] based on the extesion. PDF files require different preprocessing
    /// than images.
    fn make(&self, ext: &Ext) -> Preprocessor;
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::services::fs::{
        noop as noop_fs, tracked as tracked_fs, working as working_fs,
    };
    use crate::testingtools::services::preprocessor::{
        factory, failing, noop as noop_preprocessor, tracked, working,
    };
    use crate::testingtools::unit::create_test_shim;

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::time::Duration;

    #[test]
    fn preprocessor_is_used_to_generate_thumbnail() -> Result<()> {
        // given
        init_tracing();
        let (preprocessor_spies, preprocessor) = tracked(working());
        let factory_stub = factory(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start preprocessor

        // when
        shim.trigger_preprocessor()?;

        // then
        assert!(preprocessor_spies.preprocess_called());

        Ok(())
    }

    #[test]
    fn thumbnail_made_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_preprocessor()]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(shim.event_on_bus(&BusEvent::ThumbnailMade(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn thumbnail_generator_emits_encrypt_thumbnail_event_on_success() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_preprocessor()]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event
        shim.ignore_event()?; // ignore TextExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::EncryptThumbnail(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_preprocessor_fails() -> Result<()> {
        // given
        init_tracing();
        let (preprocessor_spies, preprocessor) = tracked(failing());
        let factory_stub = factory(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(preprocessor_spies.preprocess_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn preprocessor_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_preprocessor()]);
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
    fn failure_during_preprocessing_do_not_kill_service() -> Result<()> {
        // given
        init_tracing();
        let (preprocessor_spies1, preprocessor1) = tracked(failing());
        let (preprocessor_spies2, preprocessor2) = tracked(failing());
        let factory_stub = factory(vec![preprocessor1, preprocessor2]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, noop_fs());
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        shim.trigger_preprocessor()?;
        assert!(preprocessor_spies1.preprocess_called());

        // when
        shim.trigger_preprocessor()?;

        // then
        assert!(preprocessor_spies2.preprocess_called());

        Ok(())
    }

    #[test]
    fn when_thumbnail_encryption_failed_event_appears_filesystem_removes_thumbnail() -> Result<()> {
        // given
        init_tracing();
        let factory_stub = factory(vec![noop_preprocessor()]);
        let (fs_spies, working_fs) = tracked_fs(working_fs());
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus())?.run(factory_stub, working_fs);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_thumbnail_encryption_failure()?;

        // then
        assert!(fs_spies.rm_file_called());

        Ok(())
    }
}
