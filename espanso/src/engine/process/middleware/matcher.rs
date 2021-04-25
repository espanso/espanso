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

use log::trace;
use std::{cell::RefCell, collections::VecDeque};

use super::super::Middleware;
use crate::engine::{
  event::{
    input::{Key, Status},
    internal::{DetectedMatch, MatchesDetectedEvent},
    Event, EventType,
  },
  process::{Matcher, MatcherEvent},
};

const MAX_HISTORY: usize = 3; // TODO: get as parameter

pub struct MatcherMiddleware<'a, State> {
  matchers: &'a [&'a dyn Matcher<'a, State>],

  matcher_states: RefCell<VecDeque<Vec<State>>>,
}

impl<'a, State> MatcherMiddleware<'a, State> {
  pub fn new(matchers: &'a [&'a dyn Matcher<'a, State>]) -> Self {
    Self {
      matchers,
      matcher_states: RefCell::new(VecDeque::new()),
    }
  }
}

impl<'a, State> Middleware for MatcherMiddleware<'a, State> {
  fn name(&self) -> &'static str {
    "matcher"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if is_event_of_interest(&event.etype) {
      let mut matcher_states = self.matcher_states.borrow_mut();
      let prev_states = if !matcher_states.is_empty() {
        matcher_states.get(matcher_states.len() - 1)
      } else {
        None
      };

      if let EventType::Keyboard(keyboard_event) = &event.etype {
        // Backspace handling
        if keyboard_event.key == Key::Backspace {
          trace!("popping the last matcher state");
          matcher_states.pop_back();
          return event;
        }

        // Some keys (such as the arrow keys) prevent espanso from building
        // an accurate key buffer, so we need to invalidate it.
        if is_invalidating_key(&keyboard_event.key) {
          trace!("invalidating event detected, clearing matching state");
          matcher_states.clear();
          return event;
        }
      }

      let mut all_results = Vec::new();

      if let Some(matcher_event) = convert_to_matcher_event(&event.etype) {
        let mut new_states = Vec::new();
        for (i, matcher) in self.matchers.iter().enumerate() {
          let prev_state = prev_states.and_then(|states| states.get(i));

          let (state, results) = matcher.process(prev_state, &matcher_event);
          all_results.extend(results);

          new_states.push(state);
        }

        matcher_states.push_back(new_states);
        if matcher_states.len() > MAX_HISTORY {
          matcher_states.pop_front();
        }

        if !all_results.is_empty() {
          return Event::caused_by(
            event.source_id,
            EventType::MatchesDetected(MatchesDetectedEvent {
              matches: all_results
                .into_iter()
                .map(|result| DetectedMatch {
                  id: result.id,
                  trigger: Some(result.trigger),
                  right_separator: result.right_separator,
                  left_separator: result.left_separator,
                  args: result.args,
                })
                .collect(),
            }),
          );
        }
      }
    }

    event
  }
}

fn is_event_of_interest(event_type: &EventType) -> bool {
  match event_type {
    EventType::Keyboard(keyboard_event) => {
      if keyboard_event.status != Status::Pressed {
        // Skip non-press events
        false
      } else {
        match keyboard_event.key {
          // Skip modifier keys
          Key::Alt => false,
          Key::Shift => false,
          Key::CapsLock => false,
          Key::Meta => false,
          Key::NumLock => false,
          Key::Control => false,

          _ => true,
        }
      }
    }
    EventType::Mouse(mouse_event) => mouse_event.status == Status::Pressed,
    EventType::MatchInjected => true,
    _ => false,
  }
}

fn convert_to_matcher_event(event_type: &EventType) -> Option<MatcherEvent> {
  match event_type {
    EventType::Keyboard(keyboard_event) => Some(MatcherEvent::Key {
      key: keyboard_event.key.clone(),
      chars: keyboard_event.value.clone(),
    }),
    EventType::Mouse(_) => Some(MatcherEvent::VirtualSeparator),
    EventType::MatchInjected => Some(MatcherEvent::VirtualSeparator),
    _ => None,
  }
}

fn is_invalidating_key(key: &Key) -> bool {
  match key {
    Key::ArrowDown => true,
    Key::ArrowLeft => true,
    Key::ArrowRight => true,
    Key::ArrowUp => true,
    Key::End => true,
    Key::Home => true,
    Key::PageDown => true,
    Key::PageUp => true,
    Key::Escape => true,
    _ => false,
  }
}

// TODO: test
