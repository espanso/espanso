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

use log::error;

use super::super::Middleware;
use crate::engine::{
  event::{Event, EventType},
  process::Multiplexer,
};

pub struct MultiplexMiddleware<'a> {
  multiplexer: &'a dyn Multiplexer,
}

impl<'a> MultiplexMiddleware<'a> {
  pub fn new(multiplexer: &'a dyn Multiplexer) -> Self {
    Self { multiplexer }
  }
}

impl<'a> Middleware for MultiplexMiddleware<'a> {
  fn name(&self) -> &'static str {
    "multiplex"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::CauseCompensatedMatch(m_event) = event.etype {
      return match self.multiplexer.convert(m_event.m) {
        Some(new_event) => Event::caused_by(event.source_id, new_event),
        None => {
          error!("match multiplexing failed");
          Event::caused_by(event.source_id, EventType::NOOP)
        }
      };
    }

    event
  }
}

// TODO: test
