use std::path::PathBuf;

use crate::{cfg::Config, result::Result};

pub trait ConfigLoader {
    fn load(&self, path: PathBuf) -> Result<Config>;
    fn store(&self, cfg: &Config) -> Result<()>;
}
