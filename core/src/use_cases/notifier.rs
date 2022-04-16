use crate::result::Result;

pub trait Notifier: Sync + Send {
    fn notify(&self) -> Result<()>;
}
