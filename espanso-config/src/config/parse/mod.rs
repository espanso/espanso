use anyhow::Result;
use thiserror::Error;
use std::{convert::TryInto, path::Path};

mod yaml;

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ParsedConfig {
  pub label: Option<String>,

  // Includes
  pub includes: Option<Vec<String>>,
  pub excludes: Option<Vec<String>>,
  pub extra_includes: Option<Vec<String>>,
  pub extra_excludes: Option<Vec<String>>,
  pub use_standard_includes: Option<bool>,

  // Filters
  pub filter_title: Option<String>,
  pub filter_class: Option<String>,
  pub filter_exec: Option<String>,
  pub filter_os: Option<String>,
}

impl ParsedConfig {
  pub fn load(path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(path)?;
    match yaml::YAMLConfig::parse_from_str(&content) {
      Ok(config) => {
        Ok(config.try_into()?)
      }
      Err(err) => {
        Err(ParsedConfigError::LoadFailed(err).into())
      }
    }
  }
}

#[derive(Error, Debug)]
pub enum ParsedConfigError {
  #[error("can't load config `{0}`")]
  LoadFailed(#[from] anyhow::Error),
}