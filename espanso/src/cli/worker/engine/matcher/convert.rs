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

use espanso_config::{
  config::ConfigStore,
  matches::{
    store::{MatchSet, MatchStore},
    MatchCause,
  },
};
use espanso_match::rolling::{RollingMatch, StringMatchOptions};
use std::iter::FromIterator;

pub struct MatchConverter<'a> {
  config_store: &'a dyn ConfigStore,
  match_store: &'a dyn MatchStore,
}

impl<'a> MatchConverter<'a> {
  pub fn new(config_store: &'a dyn ConfigStore, match_store: &'a dyn MatchStore) -> Self {
    Self {
      config_store,
      match_store,
    }
  }

  // TODO: test (might need to move the conversion logic into a separate function)
  pub fn get_rolling_matches(&self) -> Vec<RollingMatch<i32>> {
    let match_set = self.global_match_set();
    let mut matches = Vec::new();

    for m in match_set.matches {
      if let MatchCause::Trigger(cause) = &m.cause {
        for trigger in cause.triggers.iter() {
          matches.push(RollingMatch::from_string(
            m.id,
            &trigger,
            &StringMatchOptions {
              case_insensitive: cause.propagate_case, 
              preserve_case_markers: cause.propagate_case, 
              left_word: cause.left_word,
              right_word: cause.right_word, 
            },
          ))
        }
      }
    }

    matches
  }

  fn global_match_set(&self) -> MatchSet {
    let paths = self.config_store.get_all_match_paths();
    self.match_store.query(&Vec::from_iter(paths.into_iter()))
  }
}
