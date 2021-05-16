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

use log::debug;

use super::super::Middleware;
use crate::engine::{event::{Event, EventType}};

pub struct ExitMiddleware {}

impl ExitMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for ExitMiddleware {
  fn name(&self) -> &'static str {
    "exit"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::ExitRequested = &event.etype {
      debug!("received ExitRequested event, dispatching exit");
      return Event::caused_by(event.source_id, EventType::Exit);
    }

    event
  }
}

// TODO: test
