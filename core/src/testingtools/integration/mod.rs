use std::path::PathBuf;

pub mod api;
pub mod app;
pub mod config;
pub mod services;

pub fn doc<S: Into<String>>(name: S) -> PathBuf {
    PathBuf::from(format!("res/{}", name.into()))
}
