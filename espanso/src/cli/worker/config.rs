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

use std::{
  collections::{HashSet},
};

use crate::engine::{dispatch::ModeProvider, process::MatchFilter};
use espanso_config::{
  config::{AppProperties, Config, ConfigStore},
  matches::store::{MatchSet, MatchStore},
};
use espanso_info::{AppInfo, AppInfoProvider};
use std::iter::FromIterator;

use super::engine::render::ConfigProvider;

pub struct ConfigManager<'a> {
  config_store: &'a dyn ConfigStore,
  match_store: &'a dyn MatchStore,
  app_info_provider: &'a dyn AppInfoProvider,
}

impl<'a> ConfigManager<'a> {
  pub fn new(
    config_store: &'a dyn ConfigStore,
    match_store: &'a dyn MatchStore,
    app_info_provider: &'a dyn AppInfoProvider,
  ) -> Self {
    Self {
      config_store,
      match_store,
      app_info_provider,
    }
  }

  pub fn active(&self) -> &'a dyn Config {
    let current_app = self.app_info_provider.get_info();
    let info = to_app_properties(&current_app);
    self.config_store.active(&info)
  }

  pub fn active_context(&self) -> (&'a dyn Config, MatchSet) {
    let config = self.active();
    let match_paths = config.match_paths();
    (config, self.match_store.query(&match_paths))
  }
}

// TODO: test
fn to_app_properties(info: &AppInfo) -> AppProperties {
  AppProperties {
    title: info.title.as_deref(),
    class: info.class.as_deref(),
    exec: info.exec.as_deref(),
  }
}

impl<'a> MatchFilter for ConfigManager<'a> {
  fn filter_active(&self, matches_ids: &[i32]) -> Vec<i32> {
    let ids_set: HashSet<i32> = HashSet::from_iter(matches_ids.iter().copied());
    let (_, match_set) = self.active_context();

    match_set
      .matches
      .iter()
      .filter(|m| ids_set.contains(&m.id))
      .map(|m| m.id)
      .collect()
  }
}

impl<'a> ConfigProvider<'a> for ConfigManager<'a> {
  fn configs(&self) -> Vec<(&'a dyn Config, MatchSet)> {
    self
      .config_store
      .configs()
      .into_iter()
      .map(|config| (config, self.match_store.query(config.match_paths())))
      .collect()
  }

  fn active(&self) -> (&'a dyn Config, MatchSet) {
    self.active_context()
  }
}

impl<'a> ModeProvider for ConfigManager<'a> {
  fn active_mode(&self) -> crate::engine::dispatch::Mode {
    let config = self.active();
    match config.backend() {
      espanso_config::config::Backend::Inject => crate::engine::dispatch::Mode::Event,
      espanso_config::config::Backend::Clipboard => crate::engine::dispatch::Mode::Clipboard,
      espanso_config::config::Backend::Auto => crate::engine::dispatch::Mode::Auto {
        clipboard_threshold: config.clipboard_threshold(),
      },
    }
  }
}

impl<'a> super::engine::executor::clipboard_injector::ClipboardParamsProvider
  for ConfigManager<'a>
{
  fn get(&self) -> super::engine::executor::clipboard_injector::ClipboardParams {
    let active = self.active();
    super::engine::executor::clipboard_injector::ClipboardParams {
      pre_paste_delay: active.pre_paste_delay(),
      paste_shortcut_event_delay: 5, // TODO: read from config
      paste_shortcut: None, // TODO: read from config
      disable_x11_fast_inject: false, // TODO: read from config
    }
  }
}
