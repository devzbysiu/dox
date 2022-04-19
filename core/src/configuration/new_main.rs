#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::data_providers::config_loader::FsConfigLoader;
use crate::data_providers::config_resolver::FsConfigResolver;
use crate::data_providers::event::{DefaultEmitter, FsSink};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::notifier::WsNotifier;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::repository::TantivyRepository;
use crate::data_providers::server::{all_thumbnails, receive_document, search};
use crate::telemetry::init_tracing;
use crate::use_cases::config_resolver::ConfigResolver;
use crate::use_cases::indexer::Indexer;
use crate::use_cases::indexing_trigger::IndexingTrigger;

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

    let fs_sink = FsSink::new();
    let emitter = DefaultEmitter::new();
    let trigger = IndexingTrigger::new(Box::new(fs_sink), Box::new(emitter));

    trigger.run();

    let fs_sink = Box::new(FsSink::new());
    let notifier = Box::new(WsNotifier);
    let preprocessor_factory = Box::new(PreprocessorFactoryImpl);
    let extractor_factory = Box::new(ExtractorFactoryImpl);
    let repository = Box::new(TantivyRepository);

    let indexer = Indexer::new(
        fs_sink,
        notifier,
        preprocessor_factory,
        extractor_factory,
        repository.clone(),
    );

    indexer.run(cfg.clone());

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
