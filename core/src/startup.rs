#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::configuration::factories::Runtime;
use crate::data_providers::server::{
    all_thumbnails, document, receive_document, search, thumbnail,
};
use crate::result::SetupErr;
use crate::use_cases::cipher::CipherReader;
use crate::use_cases::services::encrypter::Encrypter;
use crate::use_cases::services::extractor::TxtExtractor;
use crate::use_cases::services::indexer::Indexer;
use crate::use_cases::services::mover::DocumentMover;
use crate::use_cases::services::thumbnailer::ThumbnailGenerator;
use crate::use_cases::services::watcher::FileWatcher;
use crate::use_cases::state::StateReader;

use rocket::{routes, Build, Rocket};
use tracing::{debug, instrument};

#[must_use]
#[instrument(skip(ctx))]
pub fn rocket(ctx: Runtime) -> Rocket<Build> {
    let fs = ctx.fs.clone();
    let cfg = ctx.cfg.clone();
    let (state_reader, cipher_reader) = setup_core(ctx).expect("failed to setup core");

    debug!("starting server...");
    rocket::build()
        .mount(
            "/",
            routes![
                search,
                thumbnail,
                all_thumbnails,
                document,
                receive_document
            ],
        )
        .manage(state_reader)
        .manage(cipher_reader)
        .manage(fs)
        .manage(cfg)
}

fn setup_core(ctx: Runtime) -> Result<(StateReader, CipherReader), SetupErr> {
    let Runtime {
        cfg,
        bus,
        fs,
        event_watcher,
        thumbnailer_factory,
        extractor_factory,
        state,
        cipher,
    } = ctx;

    let watcher = FileWatcher::new(bus.clone());
    let document_mover = DocumentMover::new(cfg.clone(), bus.clone())?;
    let thumbnail_generator = ThumbnailGenerator::new(cfg, bus.clone())?;
    let extractor = TxtExtractor::new(bus.clone())?;
    let indexer = Indexer::new(bus.clone())?;
    let encrypter = Encrypter::new(bus);

    watcher.run(event_watcher);
    document_mover.run(fs.clone());
    thumbnail_generator.run(thumbnailer_factory, fs);
    extractor.run(extractor_factory);
    indexer.run(state.writer());
    encrypter.run(cipher.writer());

    Ok((state.reader(), cipher.reader()))
}
