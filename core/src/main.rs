#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::enum_variant_names)]

use configuration::factories::{config_loader, config_resolver};

use crate::configuration::factories::Context;
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
    let path_override = env::var("DOX_CONFIG_PATH")
        .ok()
        .or_else(|| env::args().nth(1))
        .map(PathBuf::from);

    let resolver = config_resolver(config_loader());
    let cfg = resolver
        .handle_config(path_override)
        .expect("failed to get config");

    let ctx = Context::new(cfg)?;

    let _rocket = rocket(ctx).launch().await?;

    Ok(())
}
