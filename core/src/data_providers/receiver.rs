use crate::helpers::PathRefExt;
use crate::result::EventReceiverErr;
use crate::use_cases::receiver::{DocsEvent, EventReceiver};

use notify::RecommendedWatcher;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tracing::{error, warn};

pub struct FsEventReceiver {
    _watcher: RecommendedWatcher, // just keep watcher alive
    watcher_rx: Receiver<DebouncedEvent>,
}

impl FsEventReceiver {
    pub fn new<P: AsRef<Path>>(watched_dir: P) -> Result<Self, EventReceiverErr> {
        let (watcher_tx, watcher_rx) = channel();
        let mut watcher = watcher(watcher_tx, Duration::from_millis(100))?;
        watcher.watch(watched_dir, RecursiveMode::Recursive)?;
        Ok(Self {
            _watcher: watcher,
            watcher_rx,
        })
    }
}

impl EventReceiver for FsEventReceiver {
    fn recv(&self) -> Result<DocsEvent, EventReceiverErr> {
        match self.watcher_rx.recv() {
            Ok(DebouncedEvent::Create(path)) if path.is_file() && path.is_in_user_dir() => {
                Ok(DocsEvent::Created(path.into()))
            }
            Ok(e) => {
                warn!("this FS event is not supported: {:?}", e);
                Ok(DocsEvent::Other)
            }
            Err(e) => {
                error!("watch error: {:?}", e);
                Err(EventReceiverErr::ReceiveError(e))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use claim::{assert_ok, assert_ok_eq};
    use fake::faker::filesystem::en::{FileName, FilePath};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use std::fs::{self, create_dir_all};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn receiver_is_created_without_issues() {
        // given
        let watched_dir: PathBuf = FilePath().fake();

        // when
        let receiver = FsEventReceiver::new(watched_dir);

        // then
        assert_ok!(receiver);
    }

    #[test]
    fn created_event_appears_when_file_is_created_in_user_dir() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let created_file: String = FileName().fake();
        let file_path = user_dir.join(created_file);

        // when
        mk_file(&file_path)?;
        let event = receiver.recv();

        // then
        assert_ok_eq!(event, DocsEvent::Created(file_path.into()));

        Ok(())
    }

    fn mk_user_dir<P: AsRef<Path>, S: Into<String>>(base_path: P, email: S) -> Result<PathBuf> {
        let base_path = base_path.as_ref();
        let email = email.into();
        let user_dir = base_path.join(base64::encode(email));
        create_dir_all(&user_dir)?;
        Ok(user_dir)
    }

    fn mk_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let content: String = Paragraph(0..2).fake();
        fs::write(path, content)?;
        Ok(())
    }
}
