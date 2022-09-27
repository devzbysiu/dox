#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::enum_variant_names)]

use crate::configuration::factories::{
    cipher, config_loader, config_resolver, event_bus, event_watcher, extractor_factory,
    persistence, preprocessor_factory, repository,
};
use crate::configuration::telemetry::init_tracing;
use crate::data_providers::server::{
    all_thumbnails, document, receive_document, search, thumbnail,
};
use crate::result::SetupErr;
use crate::use_cases::config::Config;
use crate::use_cases::repository::RepoRead;
use crate::use_cases::services::encrypter::Encrypter;
use crate::use_cases::services::extractor::TxtExtractor;
use crate::use_cases::services::indexer::Indexer;
use crate::use_cases::services::preprocessor::ThumbnailGenerator;
use crate::use_cases::services::watcher::DocsWatcher;

use rocket::{routes, Build, Rocket};
use std::env;
use tracing::{debug, instrument};
use use_cases::bus::EventBus;
use use_cases::cipher::CipherRead;

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod helpers;
mod result;
#[cfg(test)]
mod testutils;

#[rocket::main]
async fn main() -> Result<(), SetupErr> {
    init_tracing();
    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1));
    let _rocket = rocket(path_override).launch().await?;

    Ok(())
}

#[must_use]
#[instrument]
pub fn rocket(path_override: Option<String>) -> Rocket<Build> {
    let resolver = config_resolver(config_loader());

    let cfg = resolver
        .handle_config(path_override)
        .expect("failed to get config");

    let bus = event_bus().expect("failed to create bus");
    let (repo_read, cipher_read) = setup_core(&cfg, bus).expect("failed to setup core");

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
        .manage(persistence())
        .manage(cfg)
}

fn setup_core(cfg: &Config, bus: EventBus) -> Result<(RepoRead, CipherRead), SetupErr> {
    let watcher = DocsWatcher::new(bus.clone());
    let preprocessor = ThumbnailGenerator::new(cfg, bus.clone());
    let extractor = TxtExtractor::new(bus.clone());
    let indexer = Indexer::new(bus.clone());
    let encrypter = Encrypter::new(bus);

    watcher.run(event_watcher(cfg)?);
    preprocessor.run(preprocessor_factory());
    extractor.run(extractor_factory());
    let (repo_read, repo_write) = repository(cfg)?;
    indexer.run(repo_write)?;
    let (cipher_read, cipher_write) = cipher();
    encrypter.run(cipher_write)?;

    Ok((repo_read, cipher_read))
}
