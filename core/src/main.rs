#![allow(clippy::module_name_repetitions)]

use crate::configuration::factories::{config_loader, config_resolver, Runtime};
use crate::configuration::telemetry::init_tracing;
use crate::result::SetupErr;
use crate::startup::rocket;

use std::env;
use std::path::PathBuf;

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod helpers;
mod result;
mod startup;
#[cfg(test)]
mod testingtools;

#[rocket::main]
async fn main() -> Result<(), SetupErr> {
    init_tracing();
    let cfg = config_resolver(config_loader()).handle_config(path_override())?;
    let _rocket = rocket(Runtime::new(cfg)?).launch().await?;

    Ok(())
}

fn path_override() -> Option<PathBuf> {
    env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1))
        .map(PathBuf::from)
}
