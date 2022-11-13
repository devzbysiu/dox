use crate::data_providers::bus::LocalBus;
use crate::data_providers::cipher::Chacha20Poly1305Cipher;
use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::fs::LocalFs;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::receiver::FsEventReceiver;
use crate::data_providers::repository::TantivyRepository;
use crate::result::{BusErr, EventReceiverErr, RepositoryErr, SetupErr};
use crate::use_cases::bus::EventBus;
use crate::use_cases::cipher::{CipherRead, CipherWrite};
use crate::use_cases::config::{CfgLoader, CfgResolver, Config};
use crate::use_cases::fs::Fs;
use crate::use_cases::receiver::EventRecv;
use crate::use_cases::repository::{RepoRead, RepoWrite};
use crate::use_cases::services::extractor::ExtractorCreator;
use crate::use_cases::services::preprocessor::PreprocessorCreator;

use std::sync::Arc;

pub struct Context {
    pub cfg: Config,
    pub bus: EventBus,
    pub fs: Fs,
    pub event_watcher: EventRecv,
    pub preprocessor_factory: PreprocessorCreator,
    pub extractor_factory: ExtractorCreator,
    pub repo: (RepoRead, RepoWrite),
    pub cipher: (CipherRead, CipherWrite),
}

impl Context {
    pub fn new<C: AsRef<Config>>(cfg: C) -> Result<Self, SetupErr> {
        let cfg = cfg.as_ref();
        Ok(Self {
            cfg: cfg.clone(),
            bus: event_bus()?,
            fs: fs(),
            event_watcher: event_watcher(cfg)?,
            preprocessor_factory: preprocessor_factory(),
            extractor_factory: extractor_factory(),
            repo: repository(cfg)?,
            cipher: cipher(),
        })
    }
}

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

pub fn repository<C: AsRef<Config>>(cfg: &C) -> Result<(RepoRead, RepoWrite), RepositoryErr> {
    let cfg = cfg.as_ref();
    TantivyRepository::create(cfg)
}

pub fn fs() -> Fs {
    Arc::new(LocalFs)
}

pub fn cipher() -> (CipherRead, CipherWrite) {
    Chacha20Poly1305Cipher::create()
}

pub fn event_watcher(cfg: &Config) -> Result<EventRecv, EventReceiverErr> {
    let watched_dir = cfg.watched_dir.clone();
    Ok(Box::new(FsEventReceiver::new(watched_dir)?))
}
