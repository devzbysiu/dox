use crate::result::Result;

pub trait Repository {
    fn index(&self) -> Result<()>;
}
