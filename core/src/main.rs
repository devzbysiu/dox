#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work
#![allow(clippy::module_name_repetitions)]

use crate::configuration::factories::{
    bus, config_loader, config_resolver, extractor_factory, persistence, preprocessor_factory,
    repository,
};
use crate::configuration::telemetry::init_tracing;
use crate::data_providers::fs_watcher::FsWatcher;
use crate::data_providers::server::{
    all_thumbnails, document, receive_document, search, thumbnail,
};
use crate::result::Result;
use crate::use_cases::bus::Bus;
use crate::use_cases::config::Config;
use crate::use_cases::extractor::TxtExtractor;
use crate::use_cases::indexer::Indexer;
use crate::use_cases::preprocessor::ThumbnailGenerator;
use crate::use_cases::repository::RepoRead;

use rocket::{launch, routes, Build, Rocket};
use std::env;
use tracing::{debug, instrument};

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod helpers;
mod result;

#[must_use]
#[instrument]
#[launch]
pub fn launch() -> Rocket<Build> {
    init_tracing();

    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1));

    let resolver = config_resolver(config_loader());

    let cfg = resolver
        .handle_config(path_override)
        .expect("failed to get config");

    let bus = bus().expect("failed to create bus");
    let repository = setup_core(&cfg, &bus).expect("failed to setup core");

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
        .manage(repository)
        .manage(bus)
        .manage(persistence())
        .manage(cfg)
}

fn setup_core(cfg: &Config, bus: &dyn Bus) -> Result<RepoRead> {
    let watcher = FsWatcher::new(cfg, bus);
    let preprocessor = ThumbnailGenerator::new(cfg, bus);
    let extractor = TxtExtractor::new(bus);
    let indexer = Indexer::new(bus);

    watcher.run();
    preprocessor.run(preprocessor_factory());
    extractor.run(extractor_factory());
    let (repo_read, repo_write) = repository(cfg)?;
    indexer.run(repo_write);

    Ok(repo_read)
}
