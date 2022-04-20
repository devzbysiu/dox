use crate::result::Result;

pub trait Notifier: Send {
    fn notify(&self) -> Result<()>;
}
