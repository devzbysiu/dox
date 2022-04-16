use crate::entities::extension::Ext;
use crate::helpers::PathRefExt;

use std::path::PathBuf;

#[derive(Debug)]
pub enum Location {
    FileSystem(Vec<PathBuf>),
}

impl Location {
    pub fn extension(&self) -> Ext {
        if let Location::FileSystem(paths) = self {
            paths
                .iter()
                .nth(0)
                .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
                .ext()
        } else {
            panic!("unsupported location");
        }
    }
}
