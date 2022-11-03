//! Abstraction for preprocessing received document.
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::PreprocessorErr;
use crate::use_cases::bus::{BusEvent, EventBus, EventPublisher};
use crate::use_cases::config::Config;

use rayon::ThreadPoolBuilder;
use std::path::Path;
use std::thread;
use tracing::{debug, error, instrument, warn};

pub type PreprocessorCreator = Box<dyn PreprocessorFactory>;
pub type Preprocessor = Box<dyn FilePreprocessor>;

/// Generates thumbnail either for PDF file or image file when [`Event::NewDocs`] appears on the
/// bus.
///
/// Depending on the [`Location::extension`], specific preprocessor is selected (see
/// [`FilePreprocessor`]). It then calls [`FilePreprocessor::preprocess`] method.
pub struct ThumbnailGenerator {
    cfg: Config,
    bus: EventBus,
}

impl ThumbnailGenerator {
    pub fn new(cfg: Config, bus: EventBus) -> Self {
        Self { cfg, bus }
    }

    #[instrument(skip(self, preprocessor_factory))]
    pub fn run(self, preprocessor_factory: PreprocessorCreator) {
        thread::spawn(move || -> Result<(), PreprocessorErr> {
            let tp = ThreadPoolBuilder::new().num_threads(4).build()?;
            let thumbnails_dir = self.cfg.thumbnails_dir.clone();
            let sub = self.bus.subscriber();
            loop {
                match sub.recv()? {
                    BusEvent::NewDocs(loc) => {
                        debug!("NewDocs in: '{:?}', starting preprocessing", loc);
                        let extension = loc.extension();
                        let preprocessor = preprocessor_factory.make(&extension);
                        let publ = self.bus.publisher();
                        let dir = thumbnails_dir.clone();
                        tp.spawn(move || {
                            if let Err(e) = preprocess(&loc, &preprocessor, &dir, publ) {
                                error!("extraction failed: '{}'", e);
                            }
                        });
                    }
                    e => debug!("event not supported in ThumbnailGenerator: '{}'", e),
                }
            }
        });
    }
}

fn preprocess<P: AsRef<Path>>(
    loc: &Location,
    prepr: &Preprocessor,
    thumbnails_dir: P,
    mut publ: EventPublisher,
) -> Result<(), PreprocessorErr> {
    let thumbnails_dir = thumbnails_dir.as_ref();
    let thumbnail_loc = prepr.preprocess(loc, thumbnails_dir)?;
    debug!("preprocessing finished");
    publ.send(BusEvent::ThumbnailMade(thumbnail_loc.clone()))?;
    debug!("sending encryption request for: '{:?}'", thumbnail_loc);
    publ.send(BusEvent::EncryptionRequest(thumbnail_loc))?;
    Ok(())
}

/// Abstracts the process of preprocessing received document.
///
/// This happens right after the document was received. See
/// [`Indexer::run`](crate::use_cases::indexer::Indexer::run).
pub trait FilePreprocessor: Send {
    /// Take source location as the input and the parent directory for the output.
    /// Returns the final location of the preprocessing.
    fn preprocess(
        &self,
        location: &Location,
        thumbnails_dir: &Path,
    ) -> Result<Location, PreprocessorErr>;
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
    use crate::result::BusErr;
    use crate::testingtools::{create_test_shim, Spy};

    use anyhow::{anyhow, Result};
    use fake::{Fake, Faker};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn preprocessor_is_used_to_generate_thumbnail() -> Result<()> {
        // given
        init_tracing();
        let (spy, preprocessor) = PreprocessorSpy::working();
        let factory_stub = PreprocessorFactoryStub::new(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start preprocessor

        // when
        shim.trigger_preprocessor()?;

        // then
        assert!(spy.method_called());

        Ok(())
    }

    #[test]
    fn thumbnail_made_event_appears_on_success() -> Result<()> {
        // given
        init_tracing();
        let preprocessor = Box::new(NoOpPreprocessor);
        let factory_stub = PreprocessorFactoryStub::new(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(shim.event_on_bus(&BusEvent::ThumbnailMade(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn thumbnail_generator_emits_encryption_request_event_success() -> Result<()> {
        // given
        init_tracing();
        let preprocessor = NoOpPreprocessor::new();
        let factory_stub = PreprocessorFactoryStub::new(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event
        shim.ignore_event()?; // ignore TextExtracted event

        // then
        assert!(shim.event_on_bus(&BusEvent::EncryptionRequest(shim.test_location()))?);

        Ok(())
    }

    #[test]
    fn no_event_appears_when_preprocessor_fails() -> Result<()> {
        // given
        init_tracing();
        let (spy, failing_preprocessor) = PreprocessorSpy::failing();
        let factory_stub = PreprocessorFactoryStub::new(vec![failing_preprocessor]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        // when
        shim.trigger_preprocessor()?;

        shim.ignore_event()?; // ignore NewDocs event

        // then
        assert!(spy.method_called());
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn preprocessor_ignores_other_bus_events() -> Result<()> {
        // given
        init_tracing();
        let preprocessor = NoOpPreprocessor::new();
        let factory_stub = PreprocessorFactoryStub::new(vec![preprocessor]);
        let mut shim = create_test_shim()?;
        let ignored_events = [
            BusEvent::ThumbnailMade(Faker.fake()),
            BusEvent::Indexed(Faker.fake()),
            BusEvent::PipelineFinished,
        ];
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);

        // when
        shim.send_events(&ignored_events)?;

        // then
        // all events are still on the bus, no DataExtracted and EncryptionRequest emitted
        shim.no_such_events(
            &[
                // TODO: this shouldn't use specific values - any DataExtracted and EncryptionRequest
                // event (with any data) should make this test fail
                BusEvent::DataExtracted(Faker.fake()),
                BusEvent::EncryptionRequest(Faker.fake()),
            ],
            ignored_events.len(),
        )?;
        assert!(shim.no_events_on_bus());

        Ok(())
    }

    #[test]
    fn failure_during_preprocessing_do_not_kill_service() -> Result<()> {
        // given
        let (spy1, failing_prepr1) = PreprocessorSpy::failing();
        let (spy2, failing_prepr2) = PreprocessorSpy::failing();
        let factory_stub = PreprocessorFactoryStub::new(vec![failing_prepr1, failing_prepr2]);
        let mut shim = create_test_shim()?;
        ThumbnailGenerator::new(Config::default(), shim.bus()).run(factory_stub);
        thread::sleep(Duration::from_secs(1)); // allow to start extractor

        shim.trigger_preprocessor()?;
        assert!(spy1.method_called());

        // when
        shim.trigger_preprocessor()?;

        // then
        assert!(spy2.method_called());

        Ok(())
    }

    struct PreprocessorFactoryStub {
        preprocessor_stubs: Mutex<Vec<Option<Preprocessor>>>,
        current: AtomicUsize,
    }

    impl PreprocessorFactoryStub {
        // NOTE: this bizzare `Vec` of `Preprocessor`s is required because every time the
        // preprocessor is used, it's `take`n from the extractor stub. It has to be taken because
        // it's not possible to extract it from withing a `Mutex` without using `Option`. It has to
        // be inside `Mutex` because it has to be `Sync`, otherwise it won't compile. And finally,
        // it has to be taken because the trait `ExtractorFactory` is supposed to return owned value.
        fn new(preprocessor_stubs: Vec<Preprocessor>) -> Box<Self> {
            let preprocessor_stubs = preprocessor_stubs.into_iter().map(Option::Some).collect();
            Box::new(Self {
                preprocessor_stubs: Mutex::new(preprocessor_stubs),
                current: AtomicUsize::new(0),
            })
        }
    }

    impl PreprocessorFactory for PreprocessorFactoryStub {
        fn make(&self, _ext: &Ext) -> Preprocessor {
            let current = self.current.load(Ordering::SeqCst);
            let mut stubs = self.preprocessor_stubs.lock().expect("poisoned mutex");
            let preprocessor = stubs[current].take().unwrap();
            self.current.swap(current + 1, Ordering::SeqCst);
            preprocessor
        }
    }

    struct PreprocessorSpy;

    impl PreprocessorSpy {
        fn working() -> (Spy, Preprocessor) {
            let (tx, rx) = channel();
            (Spy::new(rx), WorkingPreprocessor::new(tx))
        }

        fn failing() -> (Spy, Preprocessor) {
            let (tx, rx) = channel();
            (Spy::new(rx), FailingPreprocessor::new(tx))
        }
    }

    struct WorkingPreprocessor {
        tx: Mutex<Sender<()>>,
    }

    impl WorkingPreprocessor {
        fn new(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl FilePreprocessor for WorkingPreprocessor {
        fn preprocess(
            &self,
            location: &Location,
            _thumbnails_dir: &Path,
        ) -> std::result::Result<Location, PreprocessorErr> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Ok(location.clone())
        }
    }

    struct FailingPreprocessor {
        tx: Mutex<Sender<()>>,
    }

    impl FailingPreprocessor {
        fn new(tx: Sender<()>) -> Box<Self> {
            Box::new(Self { tx: Mutex::new(tx) })
        }
    }

    impl FilePreprocessor for FailingPreprocessor {
        fn preprocess(
            &self,
            _location: &Location,
            _thumbnails_dir: &Path,
        ) -> std::result::Result<Location, PreprocessorErr> {
            self.tx
                .lock()
                .expect("poisoned mutex")
                .send(())
                .expect("failed to send message");
            Err(PreprocessorErr::BusError(BusErr::GenericError(anyhow!(
                "error"
            ))))
        }
    }

    struct NoOpPreprocessor;

    impl NoOpPreprocessor {
        fn new() -> Box<Self> {
            Box::new(Self)
        }
    }

    impl FilePreprocessor for NoOpPreprocessor {
        fn preprocess(
            &self,
            location: &Location,
            _thumbnails_dir: &Path,
        ) -> Result<Location, PreprocessorErr> {
            Ok(location.clone())
        }
    }
}
