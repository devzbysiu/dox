use crate::result::Result;
use crate::use_cases::notifier::Notifier;

pub struct WsNotifier;

impl Notifier for WsNotifier {
    fn notify(&self) -> Result<()> {
        unimplemented!();
    }
}
