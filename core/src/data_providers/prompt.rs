//! This module allows to gather configuration data from the user. It displays interactive prompt
//! in the terminal which asks the user for the data.
use crate::helpers::PathRefExt;
use crate::result::PromptErr;
use crate::use_cases::config::Config;

use inquire::{required, CustomUserError, Text};
use std::fs;
use std::path::{Path, PathBuf};

/// Shows prompt in the terminal.
///
/// The prompt will get all the fields needed in the [`Config`] struct. It uses [`inquire`] to
/// render the prompt.
pub fn show() -> Result<Config, PromptErr> {
    let config = Config::default();
    Ok(Config {
        watched_dir: watched_dir_prompt()?,
        docs_dir: docs_dir_prompt(&config)?,
        thumbnails_dir: thumbnails_dir_prompt(&config)?,
        index_dir: index_dir_prompt(&config)?,
    })
}

fn watched_dir_prompt() -> Result<PathBuf, PromptErr> {
    Ok(PathBuf::from(
        Text::new("Path to a directory you want to watch for changes:")
            .with_autocomplete(&path_autocomplete)
            .with_validator(required!())
            .prompt()?,
    ))
}

fn path_autocomplete(input: &str) -> std::result::Result<Vec<String>, CustomUserError> {
    let path = Path::new(input);
    if path.is_dir() {
        return Ok(vec![input.to_string()]);
    }
    let parent = path.parent();
    if parent.is_none() {
        return Ok(vec![]);
    }
    let dir = fs::read_dir(parent.unwrap()); // can unwrap, it's checked earlier
    Ok(dir?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| path.str().to_string())
        .filter(|path| path.contains(input))
        .collect())
}

fn docs_dir_prompt(config: &Config) -> Result<PathBuf, PromptErr> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing documents:")
            .with_autocomplete(&path_autocomplete)
            .with_default(config.thumbnails_dir.str())
            .prompt()?,
    ))
}

fn thumbnails_dir_prompt(config: &Config) -> Result<PathBuf, PromptErr> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing thumbnails:")
            .with_autocomplete(&path_autocomplete)
            .with_default(config.thumbnails_dir.str())
            .prompt()?,
    ))
}

fn index_dir_prompt(config: &Config) -> Result<PathBuf, PromptErr> {
    Ok(PathBuf::from(
        Text::new("Path to a directory for storing index files:")
            .with_autocomplete(&path_autocomplete)
            .with_default(config.index_dir.str())
            .prompt()?,
    ))
}
