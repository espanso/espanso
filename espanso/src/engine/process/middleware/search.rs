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

use std::collections::HashMap;

use super::super::Middleware;
use crate::engine::{event::{
    internal::{DetectedMatch, MatchesDetectedEvent},
    Event, EventType,
  }};

pub trait MatchProvider {
  fn get_all_matches_ids(&self) -> Vec<i32>;
}

pub struct SearchMiddleware<'a> {
  match_provider: &'a dyn MatchProvider,
}

impl<'a> SearchMiddleware<'a> {
  pub fn new(match_provider: &'a dyn MatchProvider) -> Self {
    Self { match_provider }
  }
}

impl<'a> Middleware for SearchMiddleware<'a> {
  fn name(&self) -> &'static str {
    "search"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    if let EventType::ShowSearchBar = event.etype {
      let detected_matches = Event::caused_by(
        event.source_id,
        EventType::MatchesDetected(MatchesDetectedEvent {
          matches: self
            .match_provider
            .get_all_matches_ids()
            .into_iter()
            .map(|id| DetectedMatch {
              id,
              trigger: None,
              left_separator: None,
              right_separator: None,
              args: HashMap::new(),
            })
            .collect(),
        }),
      );
      dispatch(detected_matches);

      return Event::caused_by(event.source_id, EventType::NOOP);
    }

    event
  }
}

// TODO: test
