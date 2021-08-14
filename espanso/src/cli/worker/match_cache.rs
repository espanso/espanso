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

use std::{collections::HashMap, iter::FromIterator};

use espanso_config::{
  config::ConfigStore,
  matches::{store::MatchStore, Match, MatchEffect},
};

use super::{builtin::BuiltInMatch, engine::process::middleware::match_select::MatchSummary};

pub struct MatchCache<'a> {
  cache: HashMap<i32, &'a Match>,
}

impl<'a> MatchCache<'a> {
  pub fn load(config_store: &'a dyn ConfigStore, match_store: &'a dyn MatchStore) -> Self {
    let mut cache = HashMap::new();

    let paths = config_store.get_all_match_paths();
    let global_set = match_store.query(&Vec::from_iter(paths.into_iter()));

    for m in global_set.matches {
      cache.insert(m.id, m);
    }

    Self { cache }
  }

  fn ids(&self) -> Vec<i32> {
    self.cache.keys().copied().collect()
  }
}

impl<'a> super::engine::process::middleware::render::MatchProvider<'a> for MatchCache<'a> {
  fn matches(&self) -> Vec<&'a Match> {
    self.cache.iter().map(|(_, m)| *m).collect()
  }

  fn get(&self, id: i32) -> Option<&'a Match> {
    self.cache.get(&id).map(|m| *m)
  }
}

impl<'a> espanso_engine::process::MatchInfoProvider for MatchCache<'a> {
  fn get_force_mode(&self, match_id: i32) -> Option<espanso_engine::event::effect::TextInjectMode> {
    let m = self.cache.get(&match_id)?;
    if let MatchEffect::Text(text_effect) = &m.effect {
      if let Some(force_mode) = &text_effect.force_mode {
        match force_mode {
          espanso_config::matches::TextInjectMode::Keys => {
            return Some(espanso_engine::event::effect::TextInjectMode::Keys)
          }
          espanso_config::matches::TextInjectMode::Clipboard => {
            return Some(espanso_engine::event::effect::TextInjectMode::Clipboard)
          }
        }
      }
    }

    None
  }
}

pub struct CombinedMatchCache<'a> {
  user_match_cache: &'a MatchCache<'a>,
  builtin_match_cache: HashMap<i32, &'a BuiltInMatch>,
}

pub enum MatchVariant<'a> {
  User(&'a Match),
  Builtin(&'a BuiltInMatch),
}

impl<'a> CombinedMatchCache<'a> {
  pub fn load(user_match_cache: &'a MatchCache<'a>, builtin_matches: &'a [BuiltInMatch]) -> Self {
    let mut builtin_match_cache = HashMap::new();

    for m in builtin_matches {
      builtin_match_cache.insert(m.id, m);
    }

    Self {
      builtin_match_cache,
      user_match_cache,
    }
  }

  pub fn get(&self, match_id: i32) -> Option<MatchVariant<'a>> {
    if let Some(user_match) = self.user_match_cache.cache.get(&match_id).map(|m| *m) {
      return Some(MatchVariant::User(user_match));
    }

    if let Some(builtin_match) = self.builtin_match_cache.get(&match_id).map(|m| *m) {
      return Some(MatchVariant::Builtin(builtin_match));
    }

    None
  }
}

impl<'a> super::engine::process::middleware::match_select::MatchProvider<'a>
  for CombinedMatchCache<'a>
{
  fn get_matches(&self, ids: &[i32]) -> Vec<MatchSummary<'a>> {
    ids
      .iter()
      .filter_map(|id| self.get(*id))
      .map(|m| match m {
        MatchVariant::User(m) => MatchSummary {
          id: m.id,
          label: m.description(),
          tag: m.cause_description(),
        },
        MatchVariant::Builtin(m) => MatchSummary {
          id: m.id,
          label: m.label,
          tag: m.triggers.first().map(String::as_ref),
        },
      })
      .collect()
  }
}

impl<'a> super::engine::process::middleware::multiplex::MatchProvider<'a>
  for CombinedMatchCache<'a>
{
  fn get(
    &self,
    match_id: i32,
  ) -> Option<super::engine::process::middleware::multiplex::MatchResult<'a>> {
    Some(match self.get(match_id)? {
      MatchVariant::User(m) => super::engine::process::middleware::multiplex::MatchResult::User(m),
      MatchVariant::Builtin(m) => {
        super::engine::process::middleware::multiplex::MatchResult::Builtin(m)
      }
    })
  }
}

impl<'a> espanso_engine::process::MatchProvider for CombinedMatchCache<'a> {
  fn get_all_matches_ids(&self) -> Vec<i32> {
    let mut ids: Vec<i32> = self.builtin_match_cache.keys().copied().collect();
    ids.extend(self.user_match_cache.ids());
    ids
  }
}
