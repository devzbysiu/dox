//! Interface for loading and saving the [`Config`] structure.
//!
//! The actual place where the config will be saved to or read from is not tight to this interface
//! and it's considered to be implementation detail.
use std::path::PathBuf;

use crate::configuration::cfg::Config;
use crate::result::Result;

/// Responsible for reading/saving the configuration from/to some medium.
///
/// Used medium is the implementation detail and is not part of this interface.
pub trait ConfigLoader {
    /// Reads the configuration.
    ///
    /// This reads the configuration pointed by `PathBuf`. The `path` argument doesn't need to
    /// represent the location on the File System, this is the implementation detail.
    fn load(&self, path: PathBuf) -> Result<Config>;

    /// Saves the configuration.
    ///
    /// This saves the configuration in the place pointed by `PathBuf`. It doesn't mean that this
    /// should be saved on the disk, the medium is the detail of the implementation.
    fn store(&self, path: PathBuf, cfg: &Config) -> Result<()>;
}

/// Handles config override.
///
/// When user specifies configuration path during startup, this interface handles this case.
pub trait ConfigResolver {
    /// Loads the [`Config`] using specified path.
    ///
    /// This method should read the configuration using the path specified as an argument.
    /// If the path is `None`, then no override takes place and configuration should be loaded from
    /// original path.
    fn handle_config(&self, path_override: Option<String>) -> Result<Config>;
}
