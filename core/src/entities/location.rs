use crate::entities::extension::Ext;
use crate::helpers::PathRefExt;

use std::path::PathBuf;

#[derive(Debug)]
#[allow(unused)]
pub enum Location {
    FileSystem(Vec<PathBuf>),
}

#[allow(unused)]
impl Location {
    pub fn extension(&self) -> Ext {
        let Location::FileSystem(paths) = self;
        paths
            .get(0)
            .unwrap_or_else(|| panic!("no new paths received, this shouldn't happen"))
            .ext()
    }
}
