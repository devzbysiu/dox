use std::path::PathBuf;

use crate::configuration::cfg::Config;
use crate::result::Result;

pub trait ConfigLoader {
    fn load(&self, path: PathBuf) -> Result<Config>;
    fn store(&self, path: PathBuf, cfg: &Config) -> Result<()>;
}
