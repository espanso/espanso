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

use std::sync::Arc;

use espanso_config::config::{Config, ConfigStore};
use log::debug;

use super::PatchDefinition;

pub struct PatchedConfigStore {
  config_store: Box<dyn ConfigStore>,
  patches: Vec<PatchDefinition>,
}

impl PatchedConfigStore {
  pub fn from_store(config_store: Box<dyn ConfigStore>) -> Self {
    Self::from_store_with_patches(config_store, super::get_builtin_patches())
  }

  pub fn from_store_with_patches(
    config_store: Box<dyn ConfigStore>,
    patches: Vec<PatchDefinition>,
  ) -> Self {
    // Only keep the patches that should be active in the current system
    let active_patches = patches
      .into_iter()
      .filter(|patch| {
        let is_enabled = (patch.is_enabled)();

        if is_enabled {
          debug!("enabled '{}' patch", patch.name);
        } else {
          debug!("skipping '{}' patch", patch.name);
        }

        is_enabled
      })
      .collect();

    Self {
      config_store,
      patches: active_patches,
    }
  }
}

impl ConfigStore for PatchedConfigStore {
  fn default(&self) -> Arc<dyn Config> {
    self.config_store.default()
  }

  fn active<'f>(&'f self, app: &espanso_config::config::AppProperties) -> Arc<dyn Config> {
    let active_config = self.config_store.active(app);

    if !active_config.apply_patch() {
      return active_config;
    }

    // Check if a patch should be applied
    if let Some(patch) = self.patches.iter().find(|patch| (patch.should_patch)(app)) {
      (patch.apply)(active_config)
    } else {
      active_config
    }
  }

  fn configs(&self) -> Vec<Arc<dyn Config>> {
    self.config_store.configs()
  }

  fn get_all_match_paths(&self) -> std::collections::HashSet<String> {
    self.config_store.get_all_match_paths()
  }
}

// TODO: test
