use crate::cfg::Config;
use crate::result::Result;
use crate::use_cases::config::ConfigLoader;

use std::path::PathBuf;

pub struct FsConfigLoader;

impl ConfigLoader for FsConfigLoader {
    fn load(&self, path: PathBuf) -> Result<Config> {
        unimplemented!();
    }

    fn store(&self, path: PathBuf, config: &Config) -> Result<()> {
        unimplemented!();
    }
}
