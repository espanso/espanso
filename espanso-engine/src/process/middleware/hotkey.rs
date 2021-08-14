/*
 * This file is part of espanso id: (), trigger: (), trigger: (), left_separator: (), right_separator: (), args: () left_separator: (), right_separator: (), args: () id: (), trigger: (), left_separator: (), right_separator: (), args: ().
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

use super::super::Middleware;
use crate::event::{
  internal::{DetectedMatch, MatchesDetectedEvent},
  Event, EventType,
};

pub struct HotKeyMiddleware {}

impl HotKeyMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for HotKeyMiddleware {
  fn name(&self) -> &'static str {
    "hotkey"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::HotKey(m_event) = &event.etype {
      return Event::caused_by(
        event.source_id,
        EventType::MatchesDetected(MatchesDetectedEvent {
          matches: vec![DetectedMatch {
            id: m_event.hotkey_id,
            ..Default::default()
          }],
        }),
      );
    }

    event
  }
}

// TODO: test
