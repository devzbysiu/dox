#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::configuration::cfg::Config;
use crate::configuration::factories::{config_loader, config_resolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::fs_watcher::FsWatcher;
use crate::data_providers::notifier::WsNotifier;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::pipe::channel_pipe;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::repository::TantivyRepository;
use crate::data_providers::server::{all_thumbnails, receive_document, search};
use crate::result::Result;
use crate::telemetry::init_tracing;
use crate::use_cases::indexer::Indexer;
use crate::use_cases::repository::Repository;

use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};
use std::env;
use tracing::{debug, instrument};

use super::factories::{extractor_factory, notifier, preprocessor_factory, repository};

#[must_use]
#[instrument]
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
    FsWatcher::run(&cfg, output);

    let repository = repository(cfg)?;

    let indexer = Indexer::new(
        input,
        notifier(cfg)?,
        preprocessor_factory(),
        extractor_factory(),
        repository.clone(),
    );

    indexer.run(cfg.clone());
    Ok(repository)
}
