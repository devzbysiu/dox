#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::configuration::cfg::Config;
use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::event::FsWatcher;
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::notifier::WsNotifier;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::repository::TantivyRepository;
use crate::data_providers::server::{all_thumbnails, receive_document, search};
use crate::result::Result;
use crate::telemetry::init_tracing;
use crate::use_cases::config::ConfigResolver;
use crate::use_cases::event::channel_pipe;
use crate::use_cases::indexer::Indexer;
use crate::use_cases::repository::Repository;

use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};
use std::env;
use tracing::{debug, instrument};

#[must_use]
#[instrument]
pub fn launch() -> Rocket<Build> {
    init_tracing();

    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1));

    let resolver = FsConfigResolver::new(Box::new(FsConfigLoader));

    let cfg = resolver
        .handle_config(path_override)
        .expect("failed to get config");

    let repository = setup_core(&cfg).expect("failed to setup core");

    let fs_persistence = Box::new(FsPersistence);

    debug!("starting server...");
    rocket::build()
        .mount("/", routes![search, all_thumbnails, receive_document])
        .mount("/thumbnail", FileServer::from(&cfg.thumbnails_dir))
        .mount("/document", FileServer::from(&cfg.watched_dir))
        .manage(repository)
        .manage(fs_persistence)
        .manage(cfg)
}

fn setup_core(cfg: &Config) -> Result<Box<dyn Repository>> {
    let (input, output) = channel_pipe();
    let fs_watcher = Box::new(FsWatcher::new(&cfg, input));

    let notifier = Box::new(WsNotifier::new(&cfg)?);
    let preprocessor_factory = Box::new(PreprocessorFactoryImpl);
    let extractor_factory = Box::new(ExtractorFactoryImpl);
    let repository = Box::new(TantivyRepository::new(&cfg)?);

    let indexer = Indexer::new(
        output,
        notifier,
        preprocessor_factory,
        extractor_factory,
        repository.clone(),
    );

    indexer.run(cfg.clone());
    Ok(repository)
}
