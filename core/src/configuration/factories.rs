use crate::data_providers::config::{FsConfigLoader, FsConfigResolver};
use crate::data_providers::extractor::ExtractorFactoryImpl;
use crate::data_providers::notifier::WsNotifier;
use crate::data_providers::persistence::FsPersistence;
use crate::data_providers::preprocessor::PreprocessorFactoryImpl;
use crate::data_providers::repository::TantivyRepository;
use crate::result::Result;
use crate::use_cases::config::Config;
use crate::use_cases::config::{ConfigLoader, ConfigResolver};
use crate::use_cases::extractor::ExtractorFactory;
use crate::use_cases::notifier::Notifier;
use crate::use_cases::persistence::Persistence;
use crate::use_cases::preprocessor::PreprocessorFactory;
use crate::use_cases::repository::Repository;

pub fn config_resolver(config_loader: Box<dyn ConfigLoader>) -> Box<dyn ConfigResolver> {
    Box::new(FsConfigResolver::new(config_loader))
}

pub fn config_loader() -> Box<dyn ConfigLoader> {
    Box::new(FsConfigLoader)
}

// pub fn notifier(cfg: &Config) -> Result<Box<dyn Notifier>> {
//     Ok(Box::new(WsNotifier::new(cfg)?))
// }

pub fn preprocessor_factory() -> Box<dyn PreprocessorFactory> {
    Box::new(PreprocessorFactoryImpl)
}

pub fn extractor_factory() -> Box<dyn ExtractorFactory> {
    Box::new(ExtractorFactoryImpl)
}

pub fn repository(cfg: &Config) -> Result<Box<dyn Repository>> {
    Ok(Box::new(TantivyRepository::new(cfg)?))
}

pub fn persistence() -> Box<dyn Persistence> {
    Box::new(FsPersistence)
}
