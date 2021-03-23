/*
 * This file is part of espanso.
 *
 * C title: (), class: (), exec: ()opyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::Result;
use regex::Regex;
use std::{collections::HashMap, path::Path};

use self::config::LegacyConfig;
use crate::config::store::DefaultConfigStore;
use crate::matches::group::loader::yaml::parse::{YAMLMatch, YAMLVariable};
use crate::{
  config::Config,
  config::{AppProperties, ConfigStore},
  matches::{
    store::{MatchSet, MatchStore},
    Match, Variable,
  },
};
use std::convert::TryInto;

mod config;
mod model;

pub fn load(
  base_dir: &Path,
  package_dir: &Path,
) -> Result<(Box<dyn ConfigStore>, Box<dyn MatchStore>)> {
  let config_set = config::LegacyConfigSet::load(base_dir, package_dir)?;

  let (default_config, default_match_group) = split_config(config_set.default);
  let mut match_groups = HashMap::new();
  match_groups.insert("default".to_string(), default_match_group);

  let mut custom_configs: Vec<Box<dyn Config>> = Vec::new();
  for custom in config_set.specific {
    let (custom_config, custom_match_group) = split_config(custom);
    match_groups.insert(custom_config.name.clone(), custom_match_group);
    custom_configs.push(Box::new(custom_config));
  }

  let config_store = DefaultConfigStore::from_configs(Box::new(default_config), custom_configs)?;
  let match_store = LegacyMatchStore::new(match_groups);

  Ok((Box::new(config_store), Box::new(match_store)))
}

fn split_config(config: LegacyConfig) -> (LegacyInteropConfig, LegacyMatchGroup) {
  let global_vars = config
    .global_vars
    .iter()
    .filter_map(|var| {
      let var: YAMLVariable = serde_yaml::from_value(var.clone()).ok()?;
      let var: Variable = var.try_into().ok()?;
      Some(var)
    })
    .collect();

  let matches = config
    .matches
    .iter()
    .filter_map(|var| {
      let m: YAMLMatch = serde_yaml::from_value(var.clone()).ok()?;
      let m: Match = m.try_into().ok()?;
      Some(m)
    })
    .collect();

  let match_group = LegacyMatchGroup {
    global_vars,
    matches,
  };

  let config: LegacyInteropConfig = config.into();
  (config, match_group)
}

struct LegacyInteropConfig {
  pub name: String,

  match_paths: Vec<String>,

  filter_title: Option<Regex>,
  filter_class: Option<Regex>,
  filter_exec: Option<Regex>,
}

impl From<config::LegacyConfig> for LegacyInteropConfig {
  fn from(config: config::LegacyConfig) -> Self {
    Self {
      name: config.name.clone(),
      match_paths: vec![config.name],
      filter_title: if !config.filter_title.is_empty() {
        Regex::new(&config.filter_title).ok()
      } else {
        None
      },
      filter_class: if !config.filter_class.is_empty() {
        Regex::new(&config.filter_class).ok()
      } else {
        None
      },
      filter_exec: if !config.filter_exec.is_empty() {
        Regex::new(&config.filter_exec).ok()
      } else {
        None
      },
    }
  }
}

impl Config for LegacyInteropConfig {
  fn label(&self) -> &str {
    &self.name
  }

  fn match_paths(&self) -> &[String] {
    &self.match_paths
  }

  fn is_match(&self, app: &AppProperties) -> bool {
    if self.filter_title.is_none() && self.filter_exec.is_none() && self.filter_class.is_none() {
      return false;
    }

    let is_title_match = if let Some(title_regex) = self.filter_title.as_ref() {
      if let Some(title) = app.title {
        title_regex.is_match(title)
      } else {
        false
      }
    } else {
      true
    };

    let is_exec_match = if let Some(exec_regex) = self.filter_exec.as_ref() {
      if let Some(exec) = app.exec {
        exec_regex.is_match(exec)
      } else {
        false
      }
    } else {
      true
    };

    let is_class_match = if let Some(class_regex) = self.filter_class.as_ref() {
      if let Some(class) = app.class {
        class_regex.is_match(class)
      } else {
        false
      }
    } else {
      true
    };

    // All the filters that have been specified must be true to define a match
    is_exec_match && is_title_match && is_class_match
  }
}

struct LegacyMatchGroup {
  matches: Vec<Match>,
  global_vars: Vec<Variable>,
}

struct LegacyMatchStore {
  groups: HashMap<String, LegacyMatchGroup>,
}

impl LegacyMatchStore {
  pub fn new(groups: HashMap<String, LegacyMatchGroup>) -> Self {
    Self { groups }
  }
}

impl MatchStore for LegacyMatchStore {
  fn query(&self, paths: &[String]) -> MatchSet {
    let group = if !paths.is_empty() {
      if let Some(group) = self.groups.get(&paths[0]) {
        Some(group)
      } else {
        None
      }
    } else {
      None
    };

    if let Some(group) = group {
      MatchSet {
        matches: group.matches.iter().collect(),
        global_vars: group.global_vars.iter().collect(),
      }
    } else {
      MatchSet {
        matches: Vec::new(),
        global_vars: Vec::new(),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{fs::create_dir_all, path::Path};
  use tempdir::TempDir;

  pub fn use_test_directory(callback: impl FnOnce(&Path, &Path, &Path)) {
    let dir = TempDir::new("tempconfig").unwrap();
    let user_dir = dir.path().join("user");
    create_dir_all(&user_dir).unwrap();

    let package_dir = TempDir::new("tempconfig").unwrap();

    callback(
      &dunce::canonicalize(&dir.path()).unwrap(),
      &dunce::canonicalize(&user_dir).unwrap(),
      &dunce::canonicalize(&package_dir.path()).unwrap(),
    );
  }

  #[test]
  fn load_legacy_works_correctly() {
    use_test_directory(|base, user, packages| {
      std::fs::write(base.join("default.yml"), r#"
      backend: Clipboard

      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#).unwrap();

      std::fs::write(user.join("specific.yml"), r#"
      name: specific
      parent: default

      matches:
        - trigger: "foo"
          replace: "bar"
      "#).unwrap();

      std::fs::write(user.join("separate.yml"), r#"
      name: separate
      filter_title: "Google"

      matches:
        - trigger: "eren"
          replace: "mikasa"
      "#).unwrap();

      let (config_store, match_store) = load(base, packages).unwrap();

      let default_config = config_store.default();
      assert_eq!(default_config.match_paths().len(), 1);

      let active_config = config_store.active(&AppProperties {
        title: Some("Google"),
        class: None,
        exec: None,
      });
      assert_eq!(active_config.match_paths().len(), 1);

      let default_fallback = config_store.active(&AppProperties {
        title: Some("Yahoo"),
        class: None,
        exec: None,
      });
      assert_eq!(default_fallback.match_paths().len(), 1);

      assert_eq!(match_store.query(default_config.match_paths()).matches.len(), 2); 
      assert_eq!(match_store.query(default_config.match_paths()).global_vars.len(), 1); 
      
      assert_eq!(match_store.query(active_config.match_paths()).matches.len(), 3); 
      assert_eq!(match_store.query(active_config.match_paths()).global_vars.len(), 1); 

      assert_eq!(match_store.query(default_fallback.match_paths()).matches.len(), 2); 
      assert_eq!(match_store.query(default_fallback.match_paths()).global_vars.len(), 1); 
    });
  }

  #[test]
  fn load_legacy_with_packages() {
    use_test_directory(|base, user, packages| {
      std::fs::write(base.join("default.yml"), r#"
      backend: Clipboard

      matches:
        - trigger: "hello"
          replace: "world"
      "#).unwrap();

      create_dir_all(packages.join("test-package")).unwrap();
      std::fs::write(packages.join("test-package").join("package.yml"), r#"
      name: test-package 
      parent: default

      matches:
        - trigger: "foo"
          replace: "bar"
      "#).unwrap();

      let (config_store, match_store) = load(base, packages).unwrap();

      let default_config = config_store.default();
      assert_eq!(default_config.match_paths().len(), 1);

      assert_eq!(match_store.query(default_config.match_paths()).matches.len(), 2); 
    });
  }
}
