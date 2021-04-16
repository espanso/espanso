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

use log::{error};

use super::super::Middleware;
use crate::engine::{
  event::{
    render::RenderedEvent,
    Event,
  },
  process::{Renderer, RendererError},
};

pub struct RenderMiddleware<'a> {
  renderer: &'a dyn Renderer<'a>,
}

impl<'a> RenderMiddleware<'a> {
  pub fn new(renderer: &'a dyn Renderer<'a>) -> Self {
    Self { renderer }
  }
}

impl<'a> Middleware for RenderMiddleware<'a> {
  fn name(&self) -> &'static str {
    "render"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let Event::RenderingRequested(m_event) = event {
      match self.renderer.render(m_event.match_id, m_event.trigger_args) {
        Ok(body) => {
          let (body, cursor_hint_back_count) = process_cursor_hint(body);

          return Event::Rendered(RenderedEvent {
            trigger: m_event.trigger,
            body,
            cursor_hint_back_count,
          });
        }
        Err(err) => {
          match err.downcast_ref::<RendererError>() {
            Some(RendererError::Aborted) => return Event::NOOP,
            _ => {
              error!("error during rendering: {}", err);
              return Event::ProcessingError("An error has occurred during rendering, please examine the logs or contact support.".to_string());
            }
          }
        }
      }
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
