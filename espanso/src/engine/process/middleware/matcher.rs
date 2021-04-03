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
use crate::engine::{event::{Event, keyboard::{Key, Status}, matches_detected::{MatchResult, MatchesDetectedEvent}}, process::{Matcher, MatcherEvent}};

const MAX_HISTORY: usize = 3; // TODO: get as parameter

pub struct MatchMiddleware<'a, State> {
  matchers: &'a [&'a dyn Matcher<'a, State>],

  matcher_states: RefCell<VecDeque<Vec<State>>>,
}

impl<'a, State> MatchMiddleware<'a, State> {
  pub fn new(matchers: &'a [&'a dyn Matcher<'a, State>]) -> Self {
    Self {
      matchers,
      matcher_states: RefCell::new(VecDeque::new()),
    }
  }
}

impl<'a, State> Middleware for MatchMiddleware<'a, State> {
  fn next(&self, event: Event, _: &dyn FnMut(Event)) -> Event {
    let mut matcher_states = self.matcher_states.borrow_mut();
    let prev_states = if !matcher_states.is_empty() {
      matcher_states.get(matcher_states.len() - 1)
    } else {
      None
    };

    if let Event::Keyboard(keyboard_event) = &event {
      // Backspace handling
      if keyboard_event.key == Key::Backspace {
        trace!("popping the last matcher state");
        matcher_states.pop_back();
        return event;
      }

      // Some keys (such as the arrow keys) prevent espanso from building
      // an accurate key buffer, so we need to invalidate it.
      if is_invalidating_key(&keyboard_event.key) {
        matcher_states.clear();
        return event;
      }
    }

    // TODO: test if the matcher detects a word match when the states are cleared (probably not :( )

    let mut all_results = Vec::new();

    if let Some(matcher_event) = convert_to_matcher_event(&event) {
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

      println!("results: {:?}", all_results);

      if !all_results.is_empty() {
        return Event::MatchesDetected(MatchesDetectedEvent {
          results: all_results.into_iter().map(|result | {
            MatchResult {
              id: result.id,
              trigger: result.trigger,
              vars: result.vars,
            }
          }).collect()
        })
      }
    }

    event
  }
}

fn convert_to_matcher_event(event: &Event) -> Option<MatcherEvent> {
  if let Event::Keyboard(keyboard_event) = event {
    if keyboard_event.status == Status::Pressed {
      return Some(MatcherEvent::Key {
        key: keyboard_event.key.clone(),
        chars: keyboard_event.value.clone(),
      });
    }
  }

  // TODO: mouse event should act as separator

  None
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
