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
    pub fn new(cfg: Config, bus: EventBus) -> Result<Self> {
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
                    e => trace!("event not supported in DocumentMover: '{}'", e),
                }
            }
        });
    }

    #[instrument(skip(self, fs))]
    fn move_doc(&self, loc: Location, fs: &Fs) {
        debug!("NewDocs in: '{:?}', moving to correct location", loc);
        let publ = self.bus.publisher();
        let dir = self.cfg.docs_dir.clone();
        debug!("does docs dir exists?: {:?}", dir.exists());
        let fs = fs.clone();
        self.tp.spawn(move || {
            if let Err(e) = move_doc(&loc, &fs, &dir, publ) {
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
fn move_doc(loc: &Location, fs: &Fs, dir: &PathBuf, mut publ: EventPublisher) -> Result<()> {
    let Location::FS(paths) = loc;
    debug!("moving '{:?}' to '{:?}'", paths, dir);
    debug!("does dst exist?: {}", dir.exists());
    let mut dst_paths = Vec::new();
    for path in paths {
        let p: &std::path::Path = path.as_ref();
        debug!("does exist?: {}", p.exists());
        let dst_path = dir.join(path.filename());
        fs.mv_file(path, &dst_path)?;
        dst_paths.push(SafePathBuf::new(dst_path));
    }
    debug!("moving finished");
    publ.send(BusEvent::DocMoved(Location::FS(dst_paths)))?;
    Ok(())
}

#[instrument(skip(fs))]
fn remove_document(loc: &Location, fs: &Fs) -> Result<()> {
    let Location::FS(paths) = loc;
    for path in paths {
        fs.rm_file(path)?;
        debug!("removed '{}'", path);
    }
    debug!("document removed");
    Ok(())
}
