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

use super::{Event, Matcher, Middleware, Processor, middleware::matcher::MatchMiddleware};
use std::collections::VecDeque;

pub struct DefaultProcessor {
  event_queue: VecDeque<Event>,
  middleware: Vec<Box<dyn Middleware>>,
}

impl DefaultProcessor {
  pub fn new<MatcherState: 'static>(matchers: Vec<Box<dyn Matcher<MatcherState>>>) -> Self {
    Self {
      event_queue: VecDeque::new(),
      middleware: vec![
        Box::new(MatchMiddleware::new(matchers)),
      ]
    }
  }

  fn process_one(&mut self) -> Option<Event> {
    if let Some(event) = self.event_queue.pop_back() {
      let mut current_event = event;
      
      let mut current_queue = VecDeque::new();
      let dispatch = |event: Event| {
        // TODO: add tracing information
        current_queue.push_front(event);
      };

      for middleware in self.middleware.iter() {
        // TODO: add tracing information
        current_event = middleware.next(current_event, &dispatch);
      }

      while let Some(event) = current_queue.pop_back() {
        self.event_queue.push_front(event);
      }

      Some(current_event)
    } else {
      None
    }
  }
}

impl Processor for DefaultProcessor {
  fn process(&mut self, event: Event) -> Vec<Event> {
    self.event_queue.push_front(event);

    let mut processed_events = Vec::new();

    while !self.event_queue.is_empty() {
      if let Some(event) = self.process_one() {
        processed_events.push(event);
      }
    }

    processed_events
  }
}