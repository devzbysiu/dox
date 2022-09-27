use std::sync::Arc;

use crate::data_providers::bus::LocalBus;
use crate::data_providers::cipher::Chacha20Poly1305Cipher;
use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::receiver::FsEventReceiver;
use crate::data_providers::repository::TantivyRepository;
use crate::result::{BusErr, EventReceiverErr, RepositoryErr};
use crate::use_cases::bus::EventBus;
use crate::use_cases::cipher::{CipherRead, CipherWrite};
use crate::use_cases::config::{CfgLoader, CfgResolver, Config};
use crate::use_cases::persistence::Persistence;
use crate::use_cases::receiver::EventRecv;
use crate::use_cases::repository::{RepoRead, RepoWrite};
use crate::use_cases::services::extractor::ExtractorCreator;
use crate::use_cases::services::preprocessor::PreprocessorCreator;

pub fn config_resolver(config_loader: CfgLoader) -> CfgResolver {
    Box::new(FsConfigResolver::new(config_loader))
}

pub fn config_loader() -> CfgLoader {
    Box::new(FsConfigLoader)
}

pub fn event_bus() -> Result<EventBus, BusErr> {
    Ok(Arc::new(LocalBus::new()?))
}

pub fn preprocessor_factory() -> PreprocessorCreator {
    Box::new(PreprocessorFactoryImpl)
}

pub fn extractor_factory() -> ExtractorCreator {
    Box::new(ExtractorFactoryImpl)
}

pub fn repository(cfg: &Config) -> Result<(RepoRead, RepoWrite), RepositoryErr> {
    TantivyRepository::create(cfg)
}

pub fn persistence() -> Persistence {
    Box::new(FsPersistence)
}

pub fn cipher() -> (CipherRead, CipherWrite) {
    Chacha20Poly1305Cipher::create()
}

pub fn event_watcher(cfg: &Config) -> Result<EventRecv, EventReceiverErr> {
    let watched_dir = cfg.watched_dir.clone();
    Ok(Box::new(FsEventReceiver::new(watched_dir)?))
}
