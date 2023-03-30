use crate::entities::extension::Ext;
use crate::entities::location::Location;
use crate::result::{BusErr, ThumbnailerErr};
use crate::testingtools::{pipe, MutexExt, Spy, Tx};
use crate::use_cases::services::thumbnailer::{
    ThumbnailMaker, Thumbnailer, ThumbnailerCreator, ThumbnailerFactory,
};

use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub fn factory(thumbnailers: Vec<Thumbnailer>) -> ThumbnailerCreator {
    ThumbnailerFactoryStub::new(thumbnailers)
}

struct ThumbnailerFactoryStub {
    thumbnailers_stubs: Mutex<Vec<Option<Thumbnailer>>>,
    current: AtomicUsize,
}

impl ThumbnailerFactoryStub {
    // NOTE: this bizzare `Vec` of `Thumbnailer`s is required because every time the
    // thumbnailer is used, it's `take`n from the thumbnailer stub. It has to be taken because
    // it's not possible to extract it from withing a `Mutex` without using `Option`. It has to
    // be inside `Mutex` because it has to be `Sync`, otherwise it won't compile. And finally,
    // it has to be taken because the trait `ThumbnailerFactory` is supposed to return owned value.
    fn new(thumbnailer_stubs: Vec<Thumbnailer>) -> Box<Self> {
        let thumbnailer_stubs = thumbnailer_stubs.into_iter().map(Option::Some).collect();
        Box::new(Self {
            thumbnailers_stubs: Mutex::new(thumbnailer_stubs),
            current: AtomicUsize::new(0),
        })
    }
}

impl ThumbnailerFactory for ThumbnailerFactoryStub {
    fn make(&self, _ext: &Ext) -> Thumbnailer {
        let current = self.current.load(Ordering::SeqCst);
        let mut stubs = self.thumbnailers_stubs.lock().expect("poisoned mutex");
        let thumbnailer = stubs[current].take().unwrap();
        self.current.swap(current + 1, Ordering::SeqCst);
        thumbnailer
    }
}

pub fn tracked(thumbnailer: Thumbnailer) -> (ThumbnailerSpies, Thumbnailer) {
    TrackedThumbnailer::wrap(thumbnailer)
}

pub struct TrackedThumbnailer {
    thumbnailer: Thumbnailer,
    thumbnailer_tx: Tx,
}

impl TrackedThumbnailer {
    fn wrap(thumbnailer: Thumbnailer) -> (ThumbnailerSpies, Thumbnailer) {
        let (thumbnailer_tx, thumbnailer_spy) = pipe();

        (
            ThumbnailerSpies::new(thumbnailer_spy),
            Box::new(Self {
                thumbnailer,
                thumbnailer_tx,
            }),
        )
    }
}

impl ThumbnailMaker for TrackedThumbnailer {
    fn mk_thumbnail(&self, location: &Location, dir: &Path) -> Result<Location, ThumbnailerErr> {
        let res = self.thumbnailer.mk_thumbnail(location, dir);
        self.thumbnailer_tx.signal();
        res
    }
}

pub struct ThumbnailerSpies {
    thumbnailer_spy: Spy,
}

impl ThumbnailerSpies {
    fn new(thumbnailer_spy: Spy) -> Self {
        Self { thumbnailer_spy }
    }

    pub fn mk_thumbnail_called(&self) -> bool {
        self.thumbnailer_spy.method_called()
    }
}

pub fn working() -> Thumbnailer {
    WorkingThumbnailer::make()
}

struct WorkingThumbnailer;

impl WorkingThumbnailer {
    fn make() -> Thumbnailer {
        Box::new(Self)
    }
}

impl ThumbnailMaker for WorkingThumbnailer {
    fn mk_thumbnail(&self, loc: &Location, _dir: &Path) -> Result<Location, ThumbnailerErr> {
        Ok(loc.clone())
    }
}

pub fn failing() -> Thumbnailer {
    FailingThumbnailer::make()
}

struct FailingThumbnailer;

impl FailingThumbnailer {
    fn make() -> Thumbnailer {
        Box::new(Self)
    }
}

impl ThumbnailMaker for FailingThumbnailer {
    fn mk_thumbnail(&self, _loc: &Location, _dir: &Path) -> Result<Location, ThumbnailerErr> {
        Err(ThumbnailerErr::Bus(BusErr::Generic(anyhow!("error"))))
    }
}

pub fn noop() -> Thumbnailer {
    NoOpThumbnailer::make()
}

struct NoOpThumbnailer;

impl NoOpThumbnailer {
    fn make() -> Thumbnailer {
        Box::new(Self)
    }
}

impl ThumbnailMaker for NoOpThumbnailer {
    fn mk_thumbnail(
        &self,
        location: &Location,
        _thumbnails_dir: &Path,
    ) -> Result<Location, ThumbnailerErr> {
        Ok(location.clone())
    }
}
