use crate::result::Result;

pub trait Notifier {
    fn notify(&self) -> Result<()>;
}
