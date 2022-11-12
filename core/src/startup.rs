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
use crate::use_cases::services::preprocessor::ThumbnailGenerator;
use crate::use_cases::services::watcher::DocsWatcher;

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

    let watcher = DocsWatcher::new(bus.clone());
    let preprocessor = ThumbnailGenerator::new(cfg, bus.clone())?;
    let extractor = TxtExtractor::new(bus.clone())?;
    let indexer = Indexer::new(bus.clone())?;
    let encrypter = Encrypter::new(bus);

    watcher.run(event_watcher);
    preprocessor.run(preprocessor_factory, fs);
    extractor.run(extractor_factory);
    indexer.run(repo.1); // TODO: get rid of those '.1' and '.0'
    encrypter.run(cipher.1);

    Ok((repo.0, cipher.0))
}
