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
use crate::{
  event::{
    internal::{DiscardBetweenEvent, MatchSelectedEvent},
    Event, EventType,
  },
  process::EventSequenceProvider,
};

pub trait MatchFilter {
  fn filter_active(&self, matches_ids: &[i32]) -> Vec<i32>;
}

pub trait MatchSelector {
  fn select(&self, matches_ids: &[i32], is_search: bool) -> Option<i32>;
}

pub struct MatchSelectMiddleware<'a> {
  match_filter: &'a dyn MatchFilter,
  match_selector: &'a dyn MatchSelector,
  event_sequence_provider: &'a dyn EventSequenceProvider,
}

impl<'a> MatchSelectMiddleware<'a> {
  pub fn new(
    match_filter: &'a dyn MatchFilter,
    match_selector: &'a dyn MatchSelector,
    event_sequence_provider: &'a dyn EventSequenceProvider,
  ) -> Self {
    Self {
      match_filter,
      match_selector,
      event_sequence_provider,
    }
  }
}

impl<'a> Middleware for MatchSelectMiddleware<'a> {
  fn name(&self) -> &'static str {
    "match_select"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    if let EventType::MatchesDetected(m_event) = event.etype {
      let matches_ids: Vec<i32> = m_event.matches.iter().map(|m| m.id).collect();

      // Find the matches that are actually valid in the current context
      let valid_ids = self.match_filter.filter_active(&matches_ids);

      return match valid_ids.len() {
        0 => Event::caused_by(event.source_id, EventType::NOOP), // No valid matches, consume the event
        1 => {
          // Only one match, no need to show a selection dialog
          let m = m_event
            .matches
            .into_iter()
            .find(|m| m.id == *valid_ids.first().unwrap());
          if let Some(m) = m {
            Event::caused_by(
              event.source_id,
              EventType::MatchSelected(MatchSelectedEvent { chosen: m }),
            )
          } else {
            error!("MatchSelectMiddleware could not find the correspondent match");
            Event::caused_by(event.source_id, EventType::NOOP)
          }
        }
        _ => {
          let start_event_id = self.event_sequence_provider.get_next_id();

          // Multiple matches, we need to ask the user which one to use
          let next_event =
            if let Some(selected_id) = self.match_selector.select(&valid_ids, m_event.is_search) {
              let m = m_event.matches.into_iter().find(|m| m.id == selected_id);
              if let Some(m) = m {
                Event::caused_by(
                  event.source_id,
                  EventType::MatchSelected(MatchSelectedEvent { chosen: m }),
                )
              } else {
                error!("MatchSelectMiddleware could not find the correspondent match");
                Event::caused_by(event.source_id, EventType::NOOP)
              }
            } else {
              debug!("MatchSelectMiddleware did not receive any match selection");
              Event::caused_by(event.source_id, EventType::NOOP)
            };

          let end_event_id = self.event_sequence_provider.get_next_id();

          // We want to prevent espanso from "stacking up" events while the search bar is open,
          // therefore we filter out all events that were generated while the search bar was open.
          // See also: https://github.com/espanso/espanso/issues/781
          dispatch(Event::caused_by(
            event.source_id,
            EventType::DiscardBetween(DiscardBetweenEvent {
              start_id: start_event_id,
              end_id: end_event_id,
            }),
          ));

          next_event
        }
      };
    }

    event
  }
}

// TODO: test
