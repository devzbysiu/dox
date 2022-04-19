use crate::cfg::Config;
use crate::result::Result;
use crate::use_cases::{config::ConfigLoader, config_resolver::ConfigResolver};

pub struct FsConfigResolver {
    config_loader: Box<dyn ConfigLoader>,
}

impl FsConfigResolver {
    pub fn new(config_loader: Box<dyn ConfigLoader>) -> Self {
        Self { config_loader }
    }
}

impl ConfigResolver for FsConfigResolver {
    fn handle_config(&self, path_override: Option<String>) -> Result<Config> {
        unimplemented!();
    }
}
