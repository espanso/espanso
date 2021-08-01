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

use espanso_config::config::{AppProperties, Config, ConfigStore};

mod config_store;
mod patches;

pub fn patch_store(store: Box<dyn ConfigStore>) -> Box<dyn ConfigStore> {
  Box::new(config_store::PatchedConfigStore::from_store(store))
}

fn get_builtin_patches() -> Vec<PatchDefinition> {
  #[cfg(target_os = "windows")]
  return vec![
    patches::win::onenote_for_windows_10::patch(),
  ];

  #[cfg(target_os = "macos")]
  return vec![];

  #[cfg(target_os = "linux")]
  return vec![
    patches::linux::libreoffice_writer_x11::patch(),
    // TODO: all the terminals registered in the legacy version + libre office
    // + firefox
  ];
}

pub struct PatchDefinition {
  pub name: &'static str,
  pub is_enabled: fn() -> bool,
  pub should_patch: fn(app: &AppProperties) -> bool,
  pub apply: fn(config: Arc<dyn Config>, name: &str) -> Arc<dyn Config>,
}