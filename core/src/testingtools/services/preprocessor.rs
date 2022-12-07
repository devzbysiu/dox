use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::{BusErr, PreprocessorErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::services::preprocessor::{
    FilePreprocessor, Preprocessor, PreprocessorCreator, PreprocessorFactory,
};

use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub fn factory(preprocessors: Vec<Preprocessor>) -> PreprocessorCreator {
    PreprocessorFactoryStub::new(preprocessors)
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

pub fn tracked(preprocessor: Preprocessor) -> (PreprocessorSpies, Preprocessor) {
    TrackedPreprocessor::wrap(preprocessor)
}

pub struct TrackedPreprocessor {
    preprocessor: Preprocessor,
    preprocess_tx: Tx,
}

impl TrackedPreprocessor {
    fn wrap(preprocessor: Preprocessor) -> (PreprocessorSpies, Preprocessor) {
        let (preprocess_tx, preprocess_spy) = pipe();

        (
            PreprocessorSpies::new(preprocess_spy),
            Box::new(Self {
                preprocessor,
                preprocess_tx,
            }),
        )
    }
}

impl FilePreprocessor for TrackedPreprocessor {
    fn preprocess(&self, location: &Location, dir: &Path) -> Result<Location, PreprocessorErr> {
        let res = self.preprocessor.preprocess(location, dir);
        self.preprocess_tx.signal();
        res
    }
}

pub struct PreprocessorSpies {
    preprocess_spy: Spy,
}

impl PreprocessorSpies {
    fn new(preprocess_spy: Spy) -> Self {
        Self { preprocess_spy }
    }

    pub fn preprocess_called(&self) -> bool {
        self.preprocess_spy.method_called()
    }
}

pub fn working() -> Preprocessor {
    WorkingPreprocessor::make()
}

struct WorkingPreprocessor;

impl WorkingPreprocessor {
    fn make() -> Preprocessor {
        Box::new(Self)
    }
}

impl FilePreprocessor for WorkingPreprocessor {
    fn preprocess(&self, loc: &Location, _dir: &Path) -> Result<Location, PreprocessorErr> {
        Ok(loc.clone())
    }
}

pub fn failing() -> Preprocessor {
    FailingPreprocessor::make()
}

struct FailingPreprocessor;

impl FailingPreprocessor {
    fn make() -> Preprocessor {
        Box::new(Self)
    }
}

impl FilePreprocessor for FailingPreprocessor {
    fn preprocess(&self, _loc: &Location, _dir: &Path) -> Result<Location, PreprocessorErr> {
        Err(PreprocessorErr::Bus(BusErr::Generic(anyhow!("error"))))
    }
}

pub fn noop() -> Preprocessor {
    NoOpPreprocessor::make()
}

struct NoOpPreprocessor;

impl NoOpPreprocessor {
    fn make() -> Preprocessor {
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
