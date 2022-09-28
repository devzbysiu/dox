#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::enum_variant_names)]

use crate::configuration::telemetry::init_tracing;
use crate::result::SetupErr;
use crate::startup::rocket;

use std::env;

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
        .or_else(|| env::args().nth(1));
    let _rocket = rocket(path_override).launch().await?;

    Ok(())
}
