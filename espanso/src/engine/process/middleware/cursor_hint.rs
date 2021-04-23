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

use super::super::Middleware;
use crate::engine::{dispatch::Mode, event::{Event, effect::CursorHintCompensationEvent, input::{Key, KeySequenceInjectRequest}, internal::RenderedEvent, text::{TextInjectMode, TextInjectRequest}}};

pub struct CursorHintMiddleware {}

impl CursorHintMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for CursorHintMiddleware {
  fn name(&self) -> &'static str {
    "cursor_hint"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    if let Event::Rendered(m_event) = event {
      let (body, cursor_hint_back_count) = process_cursor_hint(m_event.body);

      if let Some(cursor_hint_back_count) = cursor_hint_back_count {
        dispatch(Event::CursorHintCompensation(CursorHintCompensationEvent {
          cursor_hint_back_count,
        }))
      }

      // Alter the rendered event to remove the cursor hint from the body
      return Event::Rendered(RenderedEvent {
        body,
        ..m_event
      })
    }

    event
  }
}

// TODO: test
fn process_cursor_hint(body: String) -> (String, Option<usize>) {
  if let Some(index) = body.find("$|$") {
    // Convert the byte index to a char index
    let char_str = &body[0..index];
    let char_index = char_str.chars().count();
    let total_size = body.chars().count();

    // Remove the $|$ placeholder
    let body = body.replace("$|$", "");

    // Calculate the amount of rewind moves needed (LEFT ARROW).
    // Subtract also 3, equal to the number of chars of the placeholder "$|$"
    let moves = total_size - char_index - 3;
    (body, Some(moves))
  } else {
    (body, None)
  }
}

// TODO: test
