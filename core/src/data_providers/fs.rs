use crate::entities::location::SafePathBuf;
use crate::result::FsErr;
use crate::use_cases::fs::Filesystem;

pub struct LocalFs;

impl Filesystem for LocalFs {
    fn rm_file(&self, _path: &SafePathBuf) -> Result<(), FsErr> {
        unimplemented!()
    }
}
