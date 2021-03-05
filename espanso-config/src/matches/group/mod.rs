use anyhow::Result;
use std::{
  path::{Path},
};

use super::{Match, Variable};

mod loader;
mod path;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatchGroup {
  pub imports: Vec<String>,
  pub global_vars: Vec<Variable>,
  pub matches: Vec<Match>,
}

impl Default for MatchGroup {
  fn default() -> Self {
    Self {
      imports: Vec::new(),
      global_vars: Vec::new(),
      matches: Vec::new(),
    }
  }
}

impl MatchGroup {
  // TODO: test
  pub fn load(group_path: &Path) -> Result<Self> {
    loader::load_match_group(group_path)
  }
}
