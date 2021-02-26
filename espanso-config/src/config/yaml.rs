use anyhow::{private::kind::TraitKind, Result};
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::{collections::HashSet, convert::TryFrom};

use crate::{merge, util::is_yaml_empty};

use super::path::calculate_paths;

const STANDARD_INCLUDES: &[&str] = &["match/**/*.yml"];
const STANDARD_EXCLUDES: &[&str] = &["match/**/_*.yml"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLConfig {
  #[serde(default)]
  pub label: Option<String>,

  #[serde(default)]
  pub includes: Option<Vec<String>>,

  #[serde(default)]
  pub excludes: Option<Vec<String>>,

  #[serde(default)]
  pub extra_includes: Option<Vec<String>>,

  #[serde(default)]
  pub extra_excludes: Option<Vec<String>>,

  #[serde(default)]
  pub use_standard_includes: Option<bool>,

  // Filters
  #[serde(default)]
  pub filter_title: Option<String>,

  #[serde(default)]
  pub filter_class: Option<String>,

  #[serde(default)]
  pub filter_exec: Option<String>,

  #[serde(default)]
  pub filter_os: Option<String>,
}

impl YAMLConfig {
  pub fn parse_from_str(yaml: &str) -> Result<Self> {
    // Because an empty string is not valid YAML but we want to support it anyway
    if is_yaml_empty(yaml) {
      return Ok(serde_yaml::from_str(
        "arbitrary_field_that_will_not_block_the_parser: true",
      )?);
    }

    Ok(serde_yaml::from_str(yaml)?)
  }

  pub fn merge_parent(&mut self, parent: &YAMLConfig) {
    // Override the None fields with the parent's value
    merge!(
      YAMLConfig,
      self,
      parent,

      // Fields
      label,
      includes,
      excludes,
      extra_includes,
      extra_excludes,
      use_standard_includes,
      filter_title,
      filter_class,
      filter_exec,
      filter_os
    );
  }

  pub fn aggregate_includes(&self) -> HashSet<String> {
    let mut includes = HashSet::new();

    if self.use_standard_includes.is_none() || self.use_standard_includes.unwrap() {
      STANDARD_INCLUDES.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    if let Some(yaml_includes) = self.includes.as_ref() {
      yaml_includes.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    if let Some(extra_includes) = self.extra_includes.as_ref() {
      extra_includes.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    includes
  }

  pub fn aggregate_excludes(&self) -> HashSet<String> {
    let mut excludes = HashSet::new();

    if self.use_standard_includes.is_none() || self.use_standard_includes.unwrap() {
      STANDARD_EXCLUDES.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    if let Some(yaml_excludes) = self.excludes.as_ref() {
      yaml_excludes.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    if let Some(extra_excludes) = self.extra_excludes.as_ref() {
      extra_excludes.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    excludes
  }
}

// TODO: convert to TryFrom (check the matches section for an example)
impl TryFrom<YAMLConfig> for super::Config {
  type Error = anyhow::Error;

  // TODO: test
  fn try_from(yaml_config: YAMLConfig) -> Result<Self, Self::Error> {
    let includes = yaml_config.aggregate_includes();
    let excludes = yaml_config.aggregate_excludes();

    // Extract the paths
    let exclude_paths = calculate_paths(excludes.iter());
    let include_paths = calculate_paths(includes.iter());

    let match_files: Vec<String> =
      Vec::from_iter(include_paths.difference(&exclude_paths).cloned());

    Ok(Self {
      label: yaml_config.label,
      match_files,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::Config;
  use std::iter::FromIterator;
  use std::{convert::TryInto, fs::create_dir_all};
  use tempdir::TempDir;

  // fn create_config(yaml: &str) -> Result<Config> {
  //   let yaml_config: YAMLConfig = serde_yaml::from_str(yaml)?;
  //   let m: Config = yaml_config.try_into()?;
  //   Ok(m)
  // }

  #[test]
  fn aggregate_includes_empty_config() {
    assert_eq!(
      YAMLConfig::parse_from_str("").unwrap().aggregate_includes(),
      HashSet::from_iter(vec!["match/**/*.yml".to_string(),].iter().cloned())
    );
  }

  #[test]
  fn aggregate_includes_no_standard() {
    assert_eq!(
      YAMLConfig::parse_from_str("use_standard_includes: false").unwrap().aggregate_includes(),
      HashSet::new()
    );
  }

  #[test]
  fn aggregate_includes_custom_includes() {
    assert_eq!(
      YAMLConfig::parse_from_str("includes: ['custom/*.yml']")
        .unwrap()
        .aggregate_includes(),
      HashSet::from_iter(
        vec!["match/**/*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_includes_extra_includes() {
    assert_eq!(
      YAMLConfig::parse_from_str("extra_includes: ['custom/*.yml']")
        .unwrap()
        .aggregate_includes(),
      HashSet::from_iter(
        vec!["match/**/*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_includes_includes_and_extra_includes() {
    assert_eq!(
      YAMLConfig::parse_from_str("includes: ['sub/*.yml']\nextra_includes: ['custom/*.yml']")
        .unwrap()
        .aggregate_includes(),
      HashSet::from_iter(
        vec!["match/**/*.yml".to_string(), "custom/*.yml".to_string(), "sub/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_empty_config() {
    assert_eq!(
      YAMLConfig::parse_from_str("").unwrap().aggregate_excludes(),
      HashSet::from_iter(vec!["match/**/_*.yml".to_string(),].iter().cloned())
    );
  }

  #[test]
  fn aggregate_excludes_no_standard() {
    assert_eq!(
      YAMLConfig::parse_from_str("use_standard_includes: false").unwrap().aggregate_excludes(),
      HashSet::new()
    );
  }

  #[test]
  fn aggregate_excludes_custom_excludes() {
    assert_eq!(
      YAMLConfig::parse_from_str("excludes: ['custom/*.yml']")
        .unwrap()
        .aggregate_excludes(),
      HashSet::from_iter(
        vec!["match/**/_*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_extra_excludes() {
    assert_eq!(
      YAMLConfig::parse_from_str("extra_excludes: ['custom/*.yml']")
        .unwrap()
        .aggregate_excludes(),
      HashSet::from_iter(
        vec!["match/**/_*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_excludes_and_extra_excludes() {
    assert_eq!(
      YAMLConfig::parse_from_str("excludes: ['sub/*.yml']\nextra_excludes: ['custom/*.yml']")
        .unwrap()
        .aggregate_excludes(),
      HashSet::from_iter(
        vec!["match/**/_*.yml".to_string(), "custom/*.yml".to_string(), "sub/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn merge_parent_field_parent_fallback() {
    let parent = 
      YAMLConfig::parse_from_str("use_standard_includes: false").unwrap();
    let mut child = 
      YAMLConfig::parse_from_str("").unwrap();
    assert_eq!(child.use_standard_includes, None);

    child.merge_parent(&parent);
    assert_eq!(child.use_standard_includes, Some(false));
  }

  #[test]
  fn merge_parent_field_child_overwrite_parent() {
    let parent = 
      YAMLConfig::parse_from_str("use_standard_includes: true").unwrap();
    let mut child = 
      YAMLConfig::parse_from_str("use_standard_includes: false").unwrap();
    assert_eq!(child.use_standard_includes, Some(false));

    child.merge_parent(&parent);
    assert_eq!(child.use_standard_includes, Some(false));
  }

  // TODO: test conversion to Config (we need to test that the file match resolution works)
}
