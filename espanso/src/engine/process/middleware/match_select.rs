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

use log::{debug, error};

use super::super::Middleware;
use crate::engine::{
  event::{
    matches::{MatchSelectedEvent},
    Event,
  },
  process::{MatchFilter, MatchSelector},
};

pub struct MatchSelectMiddleware<'a> {
  match_filter: &'a dyn MatchFilter,
  match_selector: &'a dyn MatchSelector,
}

impl<'a> MatchSelectMiddleware<'a> {
  pub fn new(match_filter: &'a dyn MatchFilter, match_selector: &'a dyn MatchSelector) -> Self {
    Self {
      match_filter,
      match_selector,
    }
  }
}

impl<'a> Middleware for MatchSelectMiddleware<'a> {
  fn name(&self) -> &'static str {
    "match_select"
  }
  
  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let Event::MatchesDetected(m_event) = event {
      let matches_ids: Vec<i32> = m_event.matches.iter().map(|m| m.id).collect();

      // Find the matches that are actually valid in the current context
      let valid_ids = self.match_filter.filter_active(&matches_ids);

      return match valid_ids.len() {
        0 => Event::NOOP, // No valid matches, consume the event
        1 => {
          // Only one match, no need to show a selection dialog
          let m = m_event
            .matches
            .into_iter()
            .find(|m| m.id == *valid_ids.first().unwrap());
          if let Some(m) = m {
            Event::MatchSelected(MatchSelectedEvent { chosen: m })
          } else {
            error!("MatchSelectMiddleware could not find the correspondent match");
            Event::NOOP
          }
        }
        _ => {
          // Multiple matches, we need to ask the user which one to use
          if let Some(selected_id) = self.match_selector.select(&valid_ids) {
            let m = m_event
              .matches
              .into_iter()
              .find(|m| m.id == selected_id);
            if let Some(m) = m {
              Event::MatchSelected(MatchSelectedEvent { chosen: m })
            } else {
              error!("MatchSelectMiddleware could not find the correspondent match");
              Event::NOOP
            }
          } else {
            debug!("MatchSelectMiddleware did not receive any match selection");
            Event::NOOP
          }
        }
      };
    }

    event
  }
}

// TODO: test