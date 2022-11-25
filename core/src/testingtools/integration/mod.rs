use std::path::PathBuf;

pub mod api;
pub mod app;
pub mod config;
pub mod services;

pub fn doc<S: Into<String>>(name: S) -> PathBuf {
    let name = name.into();
    PathBuf::from(format!("res/{}", name))
}
