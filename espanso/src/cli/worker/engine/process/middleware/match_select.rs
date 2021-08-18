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

use espanso_engine::process::MatchSelector;
use log::error;

use crate::gui::{SearchItem, SearchUI};

const MAX_LABEL_LEN: usize = 100;

pub trait MatchProvider<'a> {
  fn get_matches(&self, ids: &[i32]) -> Vec<MatchSummary<'a>>;
}

pub struct MatchSummary<'a> {
  pub id: i32,
  pub label: &'a str,
  pub tag: Option<&'a str>,
  pub is_builtin: bool,
}

pub struct MatchSelectorAdapter<'a> {
  search_ui: &'a dyn SearchUI,
  match_provider: &'a dyn MatchProvider<'a>,
}

impl<'a> MatchSelectorAdapter<'a> {
  pub fn new(search_ui: &'a dyn SearchUI, match_provider: &'a dyn MatchProvider<'a>) -> Self {
    Self {
      search_ui,
      match_provider,
    }
  }
}

impl<'a> MatchSelector for MatchSelectorAdapter<'a> {
  fn select(&self, matches_ids: &[i32]) -> Option<i32> {
    let matches = self.match_provider.get_matches(&matches_ids);
    let search_items: Vec<SearchItem> = matches
      .into_iter()
      .map(|m| {
        let clipped_label = &m.label[..std::cmp::min(m.label.len(), MAX_LABEL_LEN)];

        SearchItem {
          id: m.id.to_string(),
          label: clipped_label.to_string(),
          tag: m.tag.map(String::from),
          is_builtin: m.is_builtin,
        }
      })
      .collect();

    match self.search_ui.show(&search_items) {
      Ok(Some(selected_id)) => match selected_id.parse::<i32>() {
        Ok(id) => Some(id),
        Err(err) => {
          error!(
            "match selector received an invalid id from SearchUI: {}",
            err
          );
          None
        }
      },
      Ok(None) => None,
      Err(err) => {
        error!("SearchUI reported an error: {}", err);
        None
      }
    }
  }
}

// TODO: test
