#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::configuration::factories::Context;
use crate::data_providers::server::{
    all_thumbnails, document, receive_document, search, thumbnail,
};
use crate::result::SetupErr;
use crate::use_cases::cipher::CipherRead;
use crate::use_cases::repository::RepoRead;
use crate::use_cases::services::encrypter::Encrypter;
use crate::use_cases::services::extractor::TxtExtractor;
use crate::use_cases::services::indexer::Indexer;
use crate::use_cases::services::mover::DocumentMover;
use crate::use_cases::services::preprocessor::ThumbnailGenerator;
use crate::use_cases::services::watcher::FileWatcher;

use rocket::{routes, Build, Rocket};
use tracing::{debug, instrument};

#[must_use]
#[instrument(skip(ctx))]
pub fn rocket(ctx: Context) -> Rocket<Build> {
    let fs = ctx.fs.clone();
    let cfg = ctx.cfg.clone();
    let (repo_read, cipher_read) = setup_core(ctx).expect("failed to setup core");

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
        .manage(repo_read)
        .manage(cipher_read)
        .manage(fs)
        .manage(cfg)
}

fn setup_core(ctx: Context) -> Result<(RepoRead, CipherRead), SetupErr> {
    let Context {
        cfg,
        bus,
        fs,
        event_watcher,
        preprocessor_factory,
        extractor_factory,
        repo,
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
    thumbnail_generator.run(preprocessor_factory, fs);
    extractor.run(extractor_factory);
    indexer.run(repo.write());
    encrypter.run(cipher.write());

    Ok((repo.read(), cipher.read()))
}
