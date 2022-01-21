use anyhow::Result;
use cooldown_buffer::cooldown_buffer;
use leptess::LepTess;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let (doc_tx, doc_rx) = cooldown_buffer(Duration::from_secs(1));
    thread::spawn(move || -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_millis(100))?;
        watcher.watch("/home/zbychu/tests/notify", RecursiveMode::Recursive)?;

        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Create(path)) => doc_tx.send(path)?,
                Ok(_) => { /* not supported */ }
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });

    loop {
        let paths = doc_rx.recv()?;
        let _paths = paths
            .par_iter()
            .map(extract_text)
            .filter_map(Result::ok)
            .map(index)
            .collect::<Vec<&PathBuf>>();
    }
}

fn extract_text<P: AsRef<Path>>(p: P) -> Result<(P, String)> {
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(p.as_ref())?;
    Ok((p, lt.get_utf8_text()?))
}

fn index<P: AsRef<Path>>((p, s): (P, String)) -> P {
    debug!("--------------------");
    debug!("{}: {}", p.as_ref().display(), s);
    p
}
