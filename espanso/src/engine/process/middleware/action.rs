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
use crate::engine::{event::{Event, keyboard::{Key, KeySequenceInjectRequest}, matches::MatchSelectedEvent, text::{TextInjectMode, TextInjectRequest}}, process::{MatchFilter, MatchSelector, Multiplexer}};

pub struct ActionMiddleware {
}

impl ActionMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for ActionMiddleware {
  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    if let Event::Rendered(m_event) = &event {
      let delete_count = m_event.trigger.len();
      let delete_sequence: Vec<_> = (0..delete_count).map(|_| Key::Backspace).collect();

      dispatch(Event::TextInject(TextInjectRequest {
        text: m_event.body.clone(),
        force_mode: Some(TextInjectMode::Keys),  // TODO: determine this one dynamically
      }));

      // This is executed before the dispatched event
      return Event::KeySequenceInject(KeySequenceInjectRequest {
        keys: delete_sequence
      })
    }

    // TODO: handle images

    event
  }
}

// TODO: test
