/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

use super::{resolve::ResolvedConfig, Config, ConfigStore, ConfigStoreError};
use anyhow::Result;
use log::{debug, error};
use std::{collections::HashSet, path::Path};

pub(crate) struct DefaultConfigStore {
  default: Box<dyn Config>,
  customs: Vec<Box<dyn Config>>,
}

impl ConfigStore for DefaultConfigStore {
  fn default(&self) -> &dyn super::Config {
    self.default.as_ref()
  }

  fn active<'a>(&'a self, app: &super::AppProperties) -> &'a dyn super::Config {
    // Find a custom config that matches or fallback to the default one
    for custom in self.customs.iter() {
      if custom.is_match(app) {
        return custom.as_ref();
      }
    }
    self.default.as_ref()
  }

  fn configs(&self) -> Vec<&dyn Config> {
    let mut configs = Vec::new();

    configs.push(self.default.as_ref());
    for custom in self.customs.iter() {
      configs.push(custom.as_ref());
    }

    configs
  }

  // TODO: test
  fn get_all_match_paths(&self) -> HashSet<String> {
    let mut paths = HashSet::new();

    paths.extend(self.default().match_paths().iter().cloned());
    for custom in self.customs.iter() {
      paths.extend(custom.match_paths().iter().cloned());
    }

    paths
  }
}

impl DefaultConfigStore {
  // TODO: test
  pub fn load(config_dir: &Path) -> Result<Self> {
    if !config_dir.is_dir() {
      return Err(ConfigStoreError::InvalidConfigDir().into());
    }

    // First get the default.yml file
    let default_file = config_dir.join("default.yml");
    if !default_file.exists() || !default_file.is_file() {
      return Err(ConfigStoreError::MissingDefault().into());
    }
    let default = ResolvedConfig::load(&default_file, None)?;
    debug!("loaded default config at path: {:?}", default_file);

    // Then the others
    let mut customs: Vec<Box<dyn Config>> = Vec::new();
    for entry in std::fs::read_dir(config_dir).map_err(ConfigStoreError::IOError)? {
      let entry = entry?;
      let config_file = entry.path();
      let extension = config_file
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

      // Additional config files are loaded best-effort
      if config_file.is_file()
        && config_file != default_file
        && (extension == "yml" || extension == "yaml")
      {
        match ResolvedConfig::load(&config_file, Some(&default)) {
          Ok(config) => {
            customs.push(Box::new(config));
            debug!("loaded config at path: {:?}", config_file);
          }
          Err(err) => {
            error!(
              "unable to load config at path: {:?}, with error: {}",
              config_file, err
            );
          }
        }
      }
    }

    Ok(Self {
      default: Box::new(default),
      customs,
    })
  }

  pub fn from_configs(default: Box<dyn Config>, customs: Vec<Box<dyn Config>>) -> Result<Self> {
    Ok(Self { default, customs })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct MockConfig {
    label: String,
    is_match: bool,
  }

  impl MockConfig {
    pub fn new(label: &str, is_match: bool) -> Self {
      Self {
        label: label.to_string(),
        is_match,
      }
    }
  }

  impl Config for MockConfig {
    fn id(&self) -> i32 {
      0
    }

    fn label(&self) -> &str {
      &self.label
    }

    fn match_paths(&self) -> &[String] {
      unimplemented!()
    }

    fn is_match(&self, _: &crate::config::AppProperties) -> bool {
      self.is_match
    }

    fn backend(&self) -> crate::config::Backend {
      unimplemented!()
    }
    
    fn clipboard_threshold(&self) -> usize {
      unimplemented!()
    }
  }

  #[test]
  fn config_store_selects_correctly() {
    let default = MockConfig::new("default", false);
    let custom1 = MockConfig::new("custom1", false);
    let custom2 = MockConfig::new("custom2", true);

    let store = DefaultConfigStore {
      default: Box::new(default),
      customs: vec![Box::new(custom1), Box::new(custom2)],
    };

    assert_eq!(store.default().label(), "default");
    assert_eq!(
      store
        .active(&crate::config::AppProperties {
          title: None,
          class: None,
          exec: None,
        })
        .label(),
      "custom2"
    );
  }

  #[test]
  fn config_store_active_fallback_to_default_if_no_match() {
    let default = MockConfig::new("default", false);
    let custom1 = MockConfig::new("custom1", false);
    let custom2 = MockConfig::new("custom2", false);

    let store = DefaultConfigStore {
      default: Box::new(default),
      customs: vec![Box::new(custom1), Box::new(custom2)],
    };

    assert_eq!(store.default().label(), "default");
    assert_eq!(
      store
        .active(&crate::config::AppProperties {
          title: None,
          class: None,
          exec: None,
        })
        .label(),
      "default"
    );
  }
}
