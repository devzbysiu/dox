use crate::data_providers::bus::LocalBus;
use crate::data_providers::cipher::Chacha20Poly1305Cipher;
use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::fs::LocalFs;
use crate::data_providers::receiver::FsEventReceiver;
use crate::data_providers::state::TantivyState;
use crate::data_providers::thumbnailer::ThumbnailerFactoryImpl;
use crate::result::{BusErr, EventReceiverErr, SetupErr, StateErr};
use crate::use_cases::bus::EventBus;
use crate::use_cases::cipher::Cipher;
use crate::use_cases::config::{CfgLoader, CfgResolver, Config};
use crate::use_cases::fs::Fs;
use crate::use_cases::receiver::EventRecv;
use crate::use_cases::services::extractor::ExtractorCreator;
use crate::use_cases::services::thumbnailer::ThumbnailerCreator;
use crate::use_cases::state::State;

use std::sync::Arc;

pub struct Runtime {
    pub cfg: Config,
    pub bus: EventBus,
    pub fs: Fs,
    pub event_watcher: EventRecv,
    pub thumbnailer_factory: ThumbnailerCreator,
    pub extractor_factory: ExtractorCreator,
    pub state: State,
    pub cipher: Cipher,
}

impl Runtime {
    pub fn new<C: AsRef<Config>>(cfg: C) -> Result<Self, SetupErr> {
        let cfg = cfg.as_ref();
        Ok(Self {
            cfg: cfg.clone(),
            bus: event_bus()?,
            fs: fs(),
            event_watcher: event_watcher(cfg)?,
            thumbnailer_factory: thumbnailer_factory(),
            extractor_factory: extractor_factory(),
            state: state(cfg)?,
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

pub fn thumbnailer_factory() -> ThumbnailerCreator {
    Box::new(ThumbnailerFactoryImpl)
}

pub fn extractor_factory() -> ExtractorCreator {
    Box::new(ExtractorFactoryImpl)
}

pub fn state<C: AsRef<Config>>(cfg: &C) -> Result<State, StateErr> {
    let cfg = cfg.as_ref();
    TantivyState::create(cfg)
}

pub fn fs() -> Fs {
    Arc::new(LocalFs)
}

pub fn cipher() -> Cipher {
    Chacha20Poly1305Cipher::create()
}

pub fn event_watcher(cfg: &Config) -> Result<EventRecv, EventReceiverErr> {
    let watched_dir = cfg.watched_dir.clone();
    Ok(Box::new(FsEventReceiver::new(watched_dir)?))
}
