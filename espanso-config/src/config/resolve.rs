use super::{parse::ParsedConfig, path::calculate_paths, Config};
use crate::merge;
use anyhow::Result;
use std::iter::FromIterator;
use std::{collections::HashSet, path::Path};
use thiserror::Error;

const STANDARD_INCLUDES: &[&str] = &["../match/**/*.yml"];
const STANDARD_EXCLUDES: &[&str] = &["../match/**/_*.yml"];

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedConfig {
  parsed: ParsedConfig,

  // Generated properties
  match_paths: HashSet<String>,
}

impl Default for ResolvedConfig {
  fn default() -> Self {
    Self {
      parsed: Default::default(),
      match_paths: HashSet::new(),
    }
  }
}

impl Config for ResolvedConfig {
  fn label(&self) -> &str {
    self.parsed.label.as_deref().unwrap_or("none")
  }

  fn match_paths(&self) -> &HashSet<String> {
    &self.match_paths
  }
}

impl ResolvedConfig {
  pub fn load(path: &Path, parent: Option<&Self>) -> Result<Self> {
    let mut config = ParsedConfig::load(path)?;

    // Merge with parent config if present
    if let Some(parent) = parent {
      Self::merge_parsed(&mut config, &parent.parsed);
    }

    // Extract the base directory
    let base_dir = path
      .parent()
      .ok_or_else(|| ResolveError::ParentResolveFailed())?;

    let match_paths = Self::generate_match_paths(&config, base_dir);

    Ok(Self {
      parsed: config,
      match_paths,
    })
  }

  fn merge_parsed(child: &mut ParsedConfig, parent: &ParsedConfig) {
    // Override the None fields with the parent's value
    merge!(
      ParsedConfig,
      child,
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

  fn aggregate_includes(config: &ParsedConfig) -> HashSet<String> {
    let mut includes = HashSet::new();

    if config.use_standard_includes.is_none() || config.use_standard_includes.unwrap() {
      STANDARD_INCLUDES.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    if let Some(yaml_includes) = config.includes.as_ref() {
      yaml_includes.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    if let Some(extra_includes) = config.extra_includes.as_ref() {
      extra_includes.iter().for_each(|include| {
        includes.insert(include.to_string());
      })
    }

    includes
  }

  fn aggregate_excludes(config: &ParsedConfig) -> HashSet<String> {
    let mut excludes = HashSet::new();

    if config.use_standard_includes.is_none() || config.use_standard_includes.unwrap() {
      STANDARD_EXCLUDES.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    if let Some(yaml_excludes) = config.excludes.as_ref() {
      yaml_excludes.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    if let Some(extra_excludes) = config.extra_excludes.as_ref() {
      extra_excludes.iter().for_each(|exclude| {
        excludes.insert(exclude.to_string());
      })
    }

    excludes
  }

  fn generate_match_paths(config: &ParsedConfig, base_dir: &Path) -> HashSet<String> {
    let includes = Self::aggregate_includes(config);
    let excludes = Self::aggregate_excludes(config);

    // Extract the paths
    let exclude_paths = calculate_paths(base_dir, excludes.iter());
    let include_paths = calculate_paths(base_dir, includes.iter());

    HashSet::from_iter(include_paths.difference(&exclude_paths).cloned())
  }
}

#[derive(Error, Debug)]
pub enum ResolveError {
  #[error("unable to resolve parent path")]
  ParentResolveFailed(),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::tests::use_test_directory;
  use std::fs::create_dir_all;
  use std::iter::FromIterator;

  #[test]
  fn aggregate_includes_empty_config() {
    assert_eq!(
      ResolvedConfig::aggregate_includes(&ParsedConfig {
        ..Default::default()
      }),
      HashSet::from_iter(vec!["../match/**/*.yml".to_string(),].iter().cloned())
    );
  }

  #[test]
  fn aggregate_includes_no_standard() {
    assert_eq!(
      ResolvedConfig::aggregate_includes(&ParsedConfig {
        use_standard_includes: Some(false),
        ..Default::default()
      }),
      HashSet::new()
    );
  }

  #[test]
  fn aggregate_includes_custom_includes() {
    assert_eq!(
      ResolvedConfig::aggregate_includes(&ParsedConfig {
        includes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_includes_extra_includes() {
    assert_eq!(
      ResolvedConfig::aggregate_includes(&ParsedConfig {
        extra_includes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_includes_includes_and_extra_includes() {
    assert_eq!(
      ResolvedConfig::aggregate_includes(&ParsedConfig {
        includes: Some(vec!["sub/*.yml".to_string()]),
        extra_includes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/*.yml".to_string(), "custom/*.yml".to_string(), "sub/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_empty_config() {
    assert_eq!(
      ResolvedConfig::aggregate_excludes(&ParsedConfig {
        ..Default::default()
      }),
      HashSet::from_iter(vec!["../match/**/_*.yml".to_string(),].iter().cloned())
    );
  }

  #[test]
  fn aggregate_excludes_no_standard() {
    assert_eq!(
      ResolvedConfig::aggregate_excludes(&ParsedConfig {
        use_standard_includes: Some(false),
        ..Default::default()
      }),
      HashSet::new()
    );
  }

  #[test]
  fn aggregate_excludes_custom_excludes() {
    assert_eq!(
      ResolvedConfig::aggregate_excludes(&ParsedConfig {
        excludes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/_*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_extra_excludes() {
    assert_eq!(
      ResolvedConfig::aggregate_excludes(&ParsedConfig {
        extra_excludes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/_*.yml".to_string(), "custom/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn aggregate_excludes_excludes_and_extra_excludes() {
    assert_eq!(
      ResolvedConfig::aggregate_excludes(&ParsedConfig {
        excludes: Some(vec!["sub/*.yml".to_string()]),
        extra_excludes: Some(vec!["custom/*.yml".to_string()]),
        ..Default::default()
      }),
      HashSet::from_iter(
        vec!["../match/**/_*.yml".to_string(), "custom/*.yml".to_string(), "sub/*.yml".to_string()]
          .iter()
          .cloned()
      )
    );
  }

  #[test]
  fn merge_parent_field_parent_fallback() {
    let parent = ParsedConfig {
      use_standard_includes: Some(false),
      ..Default::default()
    };
    let mut child = ParsedConfig {
      ..Default::default()
    };
    assert_eq!(child.use_standard_includes, None);

    ResolvedConfig::merge_parsed(&mut child, &parent);
    assert_eq!(child.use_standard_includes, Some(false));
  }

  #[test]
  fn merge_parent_field_child_overwrite_parent() {
    let parent = ParsedConfig {
      use_standard_includes: Some(true),
      ..Default::default()
    };
    let mut child = ParsedConfig {
      use_standard_includes: Some(false),
      ..Default::default()
    };
    assert_eq!(child.use_standard_includes, Some(false));

    ResolvedConfig::merge_parsed(&mut child, &parent);
    assert_eq!(child.use_standard_includes, Some(false));
  }

  #[test]
  fn match_paths_generated_correctly() {
    use_test_directory(|_, match_dir, config_dir| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();
      let another_file = match_dir.join("another.yml");
      std::fs::write(&another_file, "test").unwrap();
      let under_file = match_dir.join("_sub.yml");
      std::fs::write(&under_file, "test").unwrap();
      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(&config_file, "").unwrap();

      let config = ResolvedConfig::load(&config_file, None).unwrap();

      let mut expected = HashSet::new();
      expected.insert(base_file.to_string_lossy().to_string());
      expected.insert(another_file.to_string_lossy().to_string());
      expected.insert(sub_file.to_string_lossy().to_string());

      assert_eq!(config.match_paths(), &expected);
    });
  }

  #[test]
  fn match_paths_generated_correctly_with_child_config() {
    use_test_directory(|_, match_dir, config_dir| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();
      let another_file = match_dir.join("another.yml");
      std::fs::write(&another_file, "test").unwrap();
      let under_file = match_dir.join("_sub.yml");
      std::fs::write(&under_file, "test").unwrap();
      let sub_file = sub_dir.join("another.yml");
      std::fs::write(&sub_file, "test").unwrap();
      let sub_under_file = sub_dir.join("_sub.yml");
      std::fs::write(&sub_under_file, "test").unwrap();

      // Configs

      let parent_file = config_dir.join("parent.yml");
      std::fs::write(&parent_file, r#"
      excludes: ['../**/another.yml']
      "#).unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(&config_file, r#"
      use_standard_includes: false
      excludes: []
      includes: ["../match/sub/*.yml"]
      "#).unwrap();

      let parent = ResolvedConfig::load(&parent_file, None).unwrap();
      let child = ResolvedConfig::load(&config_file, Some(&parent)).unwrap();

      let mut expected = HashSet::new();
      expected.insert(sub_file.to_string_lossy().to_string());
      expected.insert(sub_under_file.to_string_lossy().to_string());

      assert_eq!(child.match_paths(), &expected);

      let mut expected = HashSet::new();
      expected.insert(base_file.to_string_lossy().to_string());

      assert_eq!(parent.match_paths(), &expected);
    });
  }
}
