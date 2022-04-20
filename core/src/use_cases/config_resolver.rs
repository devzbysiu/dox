use crate::configuration::cfg::Config;
use crate::result::Result;

pub trait ConfigResolver {
    fn handle_config(&self, path_override: Option<String>) -> Result<Config>;
}
