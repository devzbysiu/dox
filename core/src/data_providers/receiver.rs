use crate::entities::location::SafePathBuf;
use crate::result::EventReceiverErr;
use crate::use_cases::receiver::{DocsEvent, EventReceiver};

use notify::RecommendedWatcher;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tracing::{debug, error, warn};

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
            Ok(DebouncedEvent::Create(path)) => {
                let safepath = SafePathBuf::new(path);
                if safepath.is_file() && safepath.has_valid_ext() && safepath.is_in_user_dir() {
                    debug!("new doc detected: {safepath}");
                    Ok(DocsEvent::Created(safepath))
                } else {
                    warn!("invalid path: {safepath}");
                    Ok(DocsEvent::Other)
                }
            }
            Ok(e) => {
                warn!("this FS event is not supported in FsEventReceiver: {:?}", e);
                Ok(DocsEvent::Other)
            }
            Err(e) => {
                error!("watch error: {:?}", e);
                Err(EventReceiverErr::Receive(e))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;
    use base64::engine::general_purpose::STANDARD as b64;
    use base64::Engine;
    use claim::{assert_ok, assert_ok_eq};
    use fake::faker::filesystem::en::FileName;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::{Fake, Faker};
    use std::fs::{self, create_dir_all};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn receiver_is_created_without_issues() -> Result<()> {
        // given
        let watched_dir = tempdir()?;

        // when
        let receiver = FsEventReceiver::new(watched_dir);

        // then
        assert_ok!(receiver);

        Ok(())
    }

    #[test]
    fn created_event_appears_when_supported_file_is_created_in_user_dir() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let supported_extensions = vec!["png", "jpg", "jpeg", "webp", "pdf"];

        for extension in supported_extensions {
            let created_file: String = format!("some-file.{extension}");
            let file_path = user_dir.join(created_file);

            // when
            mk_file(&file_path)?;

            // then
            assert_ok_eq!(receiver.recv(), DocsEvent::Created(file_path.into()));
        }

        Ok(())
    }

    #[test]
    fn other_event_appears_when_file_has_unsupported_extension() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let extension: String = Faker.fake();
        let created_file: String = format!("some-file.{extension}");
        let file_path = user_dir.join(created_file);

        // when
        mk_file(file_path)?;

        // then
        assert_ok_eq!(receiver.recv(), DocsEvent::Other);

        Ok(())
    }

    fn mk_user_dir<P: AsRef<Path>, S: Into<String>>(base_path: P, email: S) -> Result<PathBuf> {
        let base_path = base_path.as_ref();
        let email = email.into();
        let user_dir = base_path.join(b64.encode(email));
        create_dir_all(&user_dir)?;
        Ok(user_dir)
    }

    fn mk_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let content: String = Paragraph(0..2).fake();
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn other_event_appears_when_directory_is_created_in_user_dir() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let created_dir: String = FileName().fake();
        let dir_path = user_dir.join(created_dir);

        // when
        mk_dir(dir_path)?;

        // then
        assert_ok_eq!(receiver.recv(), DocsEvent::Other);

        Ok(())
    }

    fn mk_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        create_dir_all(path)?;
        Ok(())
    }

    #[test]
    fn other_event_appears_when_directory_is_created_in_watched_dir() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let created_dir: String = FileName().fake();
        let dir_path = watched_dir.path().join(created_dir);

        // when
        mk_dir(dir_path)?;

        // then
        assert_ok_eq!(receiver.recv(), DocsEvent::Other);

        Ok(())
    }

    #[test]
    fn other_event_appears_when_user_dir_file_has_been_accessed() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let created_file: String = FileName().fake();
        let file_path = user_dir.join(created_file);
        mk_file(&file_path)?;
        let _event = receiver.recv(); // ignore Created event

        // when
        touch_file(file_path)?;

        // then
        assert_ok_eq!(receiver.recv(), DocsEvent::Other);

        Ok(())
    }

    fn touch_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let content: String = Paragraph(0..2).fake();
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn other_event_appears_when_user_dir_file_has_been_removed() -> Result<()> {
        // given
        let watched_dir = tempdir()?;
        let user_email: String = SafeEmail().fake();
        let user_dir = mk_user_dir(&watched_dir, user_email)?;
        let receiver = FsEventReceiver::new(&watched_dir)?;
        let created_file: String = FileName().fake();
        let file_path = user_dir.join(created_file);
        mk_file(&file_path)?;
        let _event = receiver.recv(); // ignore Created event

        // when
        rm_file(file_path)?;

        // then
        assert_ok_eq!(receiver.recv(), DocsEvent::Other);

        Ok(())
    }

    fn rm_file<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }
}
