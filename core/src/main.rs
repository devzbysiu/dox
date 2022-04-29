#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::configuration::factories::{
    config_loader, config_resolver, extractor_factory, persistence, preprocessor_factory,
    repository,
};
use crate::configuration::telemetry::init_tracing;
use crate::data_providers::fs_watcher::FsWatcher;
use crate::data_providers::server::{all_thumbnails, receive_document, search};
use crate::result::Result;
use crate::use_cases::config::Config;
use crate::use_cases::indexer::Indexer;
use crate::use_cases::repository::Repository;

use data_providers::notifier::WsNotifier;
use eventador::Eventador;
use rocket::fs::FileServer;
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

    let repository = setup_core(&cfg).expect("failed to setup core");

    debug!("starting server...");
    rocket::build()
        .mount("/", routes![search, all_thumbnails, receive_document])
        .mount("/thumbnail", FileServer::from(&cfg.thumbnails_dir))
        .mount("/document", FileServer::from(&cfg.watched_dir))
        .manage(repository)
        .manage(persistence())
        .manage(cfg)
}

fn setup_core(cfg: &Config) -> Result<Box<dyn Repository>> {
    let eventbus = Eventador::new(1024)?; // TODO: take care of this `capacity`

    FsWatcher::run(cfg, &eventbus);
    WsNotifier::run(cfg, &eventbus)?;

    let repository = repository(cfg)?;

    let indexer = Indexer::new(
        eventbus,
        preprocessor_factory(),
        extractor_factory(),
        repository.clone(),
    );

    indexer.run(cfg.clone());
    Ok(repository)
}
