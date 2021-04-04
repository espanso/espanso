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

use std::collections::HashSet;

use crate::engine::process::MatchFilter;
use espanso_config::{
  config::{AppProperties, Config, ConfigStore},
  matches::store::{MatchSet, MatchStore},
};
use espanso_info::{AppInfo, AppInfoProvider};
use std::iter::FromIterator;

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

  fn active(&self) -> &'a dyn Config {
    let current_app = self.app_info_provider.get_info();
    let info = to_app_properties(&current_app);
    self.config_store.active(&info)
  }

  fn active_match_set(&self) -> MatchSet {
    let match_paths = self.active().match_paths();
    self.match_store.query(&match_paths)
  }
}

impl<'a> MatchFilter for ConfigManager<'a> {
  fn filter_active(&self, matches_ids: &[i32]) -> Vec<i32> {
    let ids_set: HashSet<i32> = HashSet::from_iter(matches_ids.iter().copied());
    let match_set = self.active_match_set();

    match_set
      .matches
      .iter()
      .filter(|m| ids_set.contains(&m.id))
      .map(|m| m.id)
      .collect()
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
