use std::collections::HashSet;
use std::iter::FromIterator;

use super::path::calculate_paths;

const STANDARD_INCLUDES: &[&str] = &["match/**/*.yml"];
const STANDARD_EXCLUDES: &[&str] = &["match/**/_*.yml"];

#[derive(Debug, Clone)]
pub struct YAMLConfig {
  pub label: Option<String>,

  pub includes: Option<Vec<String>>,

  pub excludes: Option<Vec<String>>,

  pub extra_includes: Option<Vec<String>>,

  pub extra_excludes: Option<Vec<String>>,

  pub use_standard_includes: bool,

  // Filters

  pub filter_title: Option<String>,
  pub filter_class: Option<String>,
  pub filter_exec: Option<String>,
  pub filter_os: Option<String>,
}

impl YAMLConfig {
  // TODO test
  pub fn aggregate_includes(&self) -> HashSet<String> {
    let mut includes = HashSet::new();

    if self.use_standard_includes {
      STANDARD_INCLUDES.iter().for_each(|include| { includes.insert(include.to_string()); })
    }

    if let Some(yaml_includes) = self.includes.as_ref() {
      yaml_includes.iter().for_each(|include| { includes.insert(include.to_string()); })
    }

    if let Some(extra_includes) = self.extra_includes.as_ref() {
      extra_includes.iter().for_each(|include| { includes.insert(include.to_string()); })
    }

    includes
  }

  // TODO test
  pub fn aggregate_excludes(&self) -> HashSet<String> {
    let mut excludes = HashSet::new();

    if self.use_standard_includes {
      STANDARD_EXCLUDES.iter().for_each(|exclude| { excludes.insert(exclude.to_string()); })
    }

    if let Some(yaml_excludes) = self.excludes.as_ref() {
      yaml_excludes.iter().for_each(|exclude| { excludes.insert(exclude.to_string()); })
    }

    if let Some(extra_excludes) = self.extra_excludes.as_ref() {
      extra_excludes.iter().for_each(|exclude| { excludes.insert(exclude.to_string()); })
    }

    excludes
  }
}

// TODO: convert to TryFrom (check the matches section for an example)
impl From<YAMLConfig> for super::Config {
  // TODO: test
  fn from(yaml_config: YAMLConfig) -> Self {
    let includes = yaml_config.aggregate_includes();
    let excludes = yaml_config.aggregate_excludes();

    // Extract the paths
    let exclude_paths = calculate_paths(excludes.iter());
    let include_paths = calculate_paths(includes.iter());
    
    let match_files: Vec<String> = Vec::from_iter(include_paths.difference(&exclude_paths).cloned());

    Self {
      label: yaml_config.label,
      match_files,
    }
  }
}
