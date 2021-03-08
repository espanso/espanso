use std::{collections::HashSet, path::Path};
use thiserror::Error;
use anyhow::Result;

mod path;
mod parse;
mod util;
mod resolve;
mod store;

pub trait Config {
  fn label(&self) -> &str;
  fn match_paths(&self) -> &[String];

  fn is_match(&self, app: &AppProperties) -> bool;
}

pub trait ConfigStore {
  fn default(&self) -> &dyn Config;
  fn active<'a>(&'a self, app: &AppProperties) -> &'a dyn Config;

  fn get_all_match_paths(&self) -> HashSet<String>;
}

pub struct AppProperties<'a> {
  pub title: Option<&'a str>,
  pub class: Option<&'a str>,
  pub exec: Option<&'a str>,
}

pub fn load_store(config_dir: &Path) -> Result<impl ConfigStore> {
  store::DefaultConfigStore::load(config_dir)
}

#[derive(Error, Debug)]
pub enum ConfigStoreError {
  #[error("invalid config directory")]
  InvalidConfigDir(),

  #[error("missing default.yml config")]
  MissingDefault(),

  #[error("io error")]
  IOError(#[from] std::io::Error),
}