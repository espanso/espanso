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

use std::cell::RefCell;

use log::trace;

use super::super::Middleware;
use crate::engine::{event::{Event, EventType, SourceId}};

/// This middleware discards all events that have a source_id smaller than its
/// configured threshold. This useful to discard past events that might have
/// been stuck in the event queue for too long.
pub struct PastEventsDiscardMiddleware {
  source_id_threshold: RefCell<SourceId>,
}

impl PastEventsDiscardMiddleware {
  pub fn new() -> Self {
    Self {
      source_id_threshold: RefCell::new(0),
    }
  }
}

impl Middleware for PastEventsDiscardMiddleware {
  fn name(&self) -> &'static str {
    "past_discard"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    let mut source_id_threshold = self.source_id_threshold.borrow_mut();
    
    // Filter out previous events
    if event.source_id < *source_id_threshold {
      trace!("discarding previous event: {:?}", event);
      return Event::caused_by(event.source_id, EventType::NOOP);
    }

    // Update the minimum threshold
    if let EventType::DiscardPrevious(m_event) = &event.etype {
      trace!("updating minimum source id threshold for events to: {}", m_event.minimum_source_id);
      *source_id_threshold = m_event.minimum_source_id;
    }

    event
  }
}

// TODO: test
