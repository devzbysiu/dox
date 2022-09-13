use crate::data_providers::bus::LocalBus;
use crate::data_providers::cipher::Chacha20Poly1305Cipher;
use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::repository::TantivyRepository;
use crate::result::Result;
use crate::use_cases::bus::EventBus;
use crate::use_cases::cipher::{CipherRead, CipherWrite};
use crate::use_cases::config::{CfgLoader, CfgResolver, Config};
use crate::use_cases::extractor::ExtractorCreator;
use crate::use_cases::persistence::Persistence;
use crate::use_cases::preprocessor::PreprocessorCreator;
use crate::use_cases::repository::{RepoRead, RepoWrite};

pub fn config_resolver(config_loader: CfgLoader) -> CfgResolver {
    Box::new(FsConfigResolver::new(config_loader))
}

pub fn config_loader() -> CfgLoader {
    Box::new(FsConfigLoader)
}

pub fn bus() -> Result<EventBus> {
    Ok(Box::new(LocalBus::new()?))
}

pub fn preprocessor_factory() -> PreprocessorCreator {
    Box::new(PreprocessorFactoryImpl)
}

pub fn extractor_factory() -> ExtractorCreator {
    Box::new(ExtractorFactoryImpl)
}

pub fn repository(cfg: &Config) -> Result<(RepoRead, RepoWrite)> {
    TantivyRepository::create(cfg)
}

pub fn persistence() -> Persistence {
    Box::new(FsPersistence)
}

pub fn cipher() -> (CipherRead, CipherWrite) {
    Chacha20Poly1305Cipher::create()
}
