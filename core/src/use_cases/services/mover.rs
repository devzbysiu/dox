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
    use crate::testingtools::services::{NoOpFs, TrackedFs};
    use crate::testingtools::unit::create_test_shim;
    use crate::testingtools::TestConfig;

    use anyhow::Result;
    use std::time::Duration;

    #[test]
    fn fs_is_used_to_move_document() -> Result<()> {
        // given
        init_tracing();
        // TODO: Think about methods like `tracked_fs(no_op_fs())` or `tracked_fs(real_fs())`
        let (fs_spies, fs) = TrackedFs::wrap(NoOpFs::new());
        let mut shim = create_test_shim()?;
        DocumentMover::new(TestConfig::new()?, shim.bus())?.run(fs);
        thread::sleep(Duration::from_secs(1)); // allow DocumentMover to start

        // when
        shim.trigger_mover()?;

        // then
        assert!(fs_spies.mv_file_called());

        Ok(())
    }
}
