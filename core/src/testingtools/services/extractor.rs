use crate::entities::document::DocDetails;
use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::ExtractorErr;
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::services::extractor::{
    DataExtractor, Extractor, ExtractorCreator, ExtractorFactory,
};

use anyhow::Result;
use leptess::tesseract::TessInitError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub fn factory(extractors: Vec<Extractor>) -> ExtractorCreator {
    ExtractorFactoryStub::new(extractors)
}

struct ExtractorFactoryStub {
    extractor_stubs: Mutex<Vec<Option<Extractor>>>,
    current: AtomicUsize,
}

impl ExtractorFactoryStub {
    // NOTE: this bizzare `Vec` of `Extractor`s is required because every time the extractor is
    // used, it's `take`n from the extractor stub. It has to be taken because it's not possible
    // to extract it from withing a `Mutex` without using `Option`. It has to be inside `Mutex`
    // because it has to be `Sync`, otherwise it won't compile. And finally, it has to be taken
    // because the trait `ExtractorFactory` is supposed to return owned value.
    fn new(extractor_stubs: Vec<Extractor>) -> Box<Self> {
        let extractor_stubs = extractor_stubs.into_iter().map(Option::Some).collect();
        Box::new(Self {
            extractor_stubs: Mutex::new(extractor_stubs),
            current: AtomicUsize::new(0),
        })
    }
}

impl ExtractorFactory for ExtractorFactoryStub {
    fn make(&self, _ext: &Ext) -> Extractor {
        let current = self.current.load(Ordering::SeqCst);
        let mut stubs = self.extractor_stubs.lock().expect("poisoned mutex");
        let extractor = stubs[current].take().unwrap();
        self.current.swap(current + 1, Ordering::SeqCst);
        extractor
    }
}

pub fn tracked(extractor: Extractor) -> (ExtractorSpies, Extractor) {
    TrackedExtractor::wrap(extractor)
}

pub struct TrackedExtractor {
    extractor: Extractor,
    extract_tx: Tx,
}

impl TrackedExtractor {
    fn wrap(extractor: Extractor) -> (ExtractorSpies, Extractor) {
        let (extract_tx, extract_spy) = pipe();

        (
            ExtractorSpies::new(extract_spy),
            Box::new(Self {
                extractor,
                extract_tx,
            }),
        )
    }
}

impl DataExtractor for TrackedExtractor {
    fn extract_data(&self, location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        let res = self.extractor.extract_data(location);
        self.extract_tx.signal();
        res
    }
}

pub struct ExtractorSpies {
    extract_spy: Spy,
}

impl ExtractorSpies {
    fn new(extract_spy: Spy) -> Self {
        Self { extract_spy }
    }

    pub fn extract_called(&self) -> bool {
        self.extract_spy.method_called()
    }
}

pub fn working() -> Extractor {
    WorkingExtractor::make()
}

struct WorkingExtractor;

impl WorkingExtractor {
    fn make() -> Extractor {
        Box::new(Self)
    }
}

impl DataExtractor for WorkingExtractor {
    fn extract_data(&self, _location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        Ok(Vec::new())
    }
}

pub fn failing() -> Extractor {
    FailingExtractor::make()
}

struct FailingExtractor;

impl FailingExtractor {
    fn make() -> Box<Self> {
        Box::new(Self)
    }
}

impl DataExtractor for FailingExtractor {
    fn extract_data(&self, _location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        Err(ExtractorErr::OcrExtract(TessInitError { code: 0 }))
    }
}

pub fn noop() -> Extractor {
    NoOpExtractor::new()
}

struct NoOpExtractor;

impl NoOpExtractor {
    fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl DataExtractor for NoOpExtractor {
    fn extract_data(&self, _location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        // nothing to do
        Ok(Vec::new())
    }
}

pub fn stub(docs_details: Vec<DocDetails>) -> Extractor {
    ExtractorStub::new(docs_details)
}

struct ExtractorStub {
    docs_details: Vec<DocDetails>,
}

impl ExtractorStub {
    fn new(docs_details: Vec<DocDetails>) -> Box<Self> {
        Box::new(Self { docs_details })
    }
}

impl DataExtractor for ExtractorStub {
    fn extract_data(&self, _location: &Location) -> Result<Vec<DocDetails>, ExtractorErr> {
        // nothing to do
        Ok(self.docs_details.clone())
    }
}
