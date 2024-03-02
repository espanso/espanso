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
use crate::event::{Event, EventType, SourceId};

/// This middleware discards all events that have a `source_id` between
/// the given maximum and minimum.
/// This useful to discard past events that might have been stuck in the
/// event queue for too long, or events generated while the search bar was open.
pub struct EventsDiscardMiddleware {
  min_id_threshold: RefCell<SourceId>,
  max_id_threshold: RefCell<SourceId>,
}

impl EventsDiscardMiddleware {
  pub fn new() -> Self {
    Self {
      min_id_threshold: RefCell::new(0),
      max_id_threshold: RefCell::new(0),
    }
  }
}

impl Middleware for EventsDiscardMiddleware {
  fn name(&self) -> &'static str {
    "discard"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    let mut min_id_threshold = self.min_id_threshold.borrow_mut();
    let mut max_id_threshold = self.max_id_threshold.borrow_mut();

    // Filter out previous events
    if event.source_id < *max_id_threshold && event.source_id >= *min_id_threshold {
      trace!("discarding previous event: {:?}", event);
      return Event::caused_by(event.source_id, EventType::NOOP);
    }

    // Update the thresholds
    if let EventType::DiscardPrevious(m_event) = &event.etype {
      trace!(
        "updating discard max_id_threshold threshold for events to: {}",
        m_event.minimum_source_id
      );
      *max_id_threshold = m_event.minimum_source_id;
    } else if let EventType::DiscardBetween(m_event) = &event.etype {
      trace!(
        "updating discard thresholds for events to: max={} min={}",
        m_event.end_id,
        m_event.start_id
      );
      *max_id_threshold = m_event.end_id;
      *min_id_threshold = m_event.start_id;
    }

    event
  }
}

// TODO: test
