#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use crate::server::{all_thumbnails, receive_document, search};
use crate::setup::setup;
use crate::telemetry::init_tracing;
use crate::user_input::handle_config;

use rocket::fs::FileServer;
use rocket::{launch, routes, Build, Rocket};
use std::env;
use tracing::{debug, instrument};

mod configuration;
mod data_providers;
mod entities;
mod helpers;
mod indexer;
mod notifier;
mod prompt;
mod result;
mod server;
mod setup;
mod telemetry;
mod use_cases;
mod user_input;

#[launch]
#[must_use]
#[instrument]
pub fn launch() -> Rocket<Build> {
    init_tracing();

    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1));
    let cfg = handle_config(path_override).expect("failed to get config");

    let config = cfg.clone();
    let repo = setup(config).expect("failed to setup indexer");
    debug!("starting server...");
    rocket::build()
        .mount("/", routes![search, all_thumbnails, receive_document])
        .mount("/thumbnail", FileServer::from(&cfg.thumbnails_dir))
        .mount("/document", FileServer::from(&cfg.watched_dir))
        .manage(repo)
        .manage(cfg)
}
