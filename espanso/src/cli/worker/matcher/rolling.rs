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

use espanso_match::rolling::{matcher::RollingMatcher, RollingMatch};

use crate::engine::{
  event::keyboard::Key,
  process::{MatchResult, Matcher, MatcherEvent},
};

use super::MatcherState;

pub struct RollingMatcherAdapter {
  matcher: RollingMatcher<i32>,
}

impl RollingMatcherAdapter {
  pub fn new(matches: &[RollingMatch<i32>]) -> Self {
    let matcher = RollingMatcher::new(matches, Default::default());

    Self { matcher }
  }
}

impl <'a> Matcher<'a, MatcherState<'a>> for RollingMatcherAdapter {
  fn process(
    &'a self,
    prev_state: Option<&MatcherState<'a>>,
    event: &MatcherEvent,
  ) -> (MatcherState<'a>, Vec<MatchResult>) {
    use espanso_match::Matcher;

    let prev_state = prev_state.map(|state| {
      if let Some(state) = state.as_rolling() {
        state
      } else {
        panic!("invalid state type received in RollingMatcherAdapter")
      }
    });
    let event = event.into();

    let (state, results) = self.matcher.process(prev_state, event);

    let enum_state = MatcherState::Rolling(state);
    let results: Vec<MatchResult> = results.into_iter().map(|result| result.into()).collect();

    (enum_state, results)
  }
}

impl From<&MatcherEvent> for espanso_match::event::Event {
  fn from(event: &MatcherEvent) -> Self {
    match event {
      MatcherEvent::Key { key, chars } => espanso_match::event::Event::Key {
        key: key.clone().into(),
        chars: chars.to_owned(),
      },
      MatcherEvent::VirtualSeparator => espanso_match::event::Event::VirtualSeparator,
    }
  }
}

impl From<espanso_match::MatchResult<i32>> for MatchResult {
  fn from(result: espanso_match::MatchResult<i32>) -> Self {
    Self {
      id: result.id,
      trigger: result.trigger, 
      vars: result.vars, 
    }
  }
}

impl From<Key> for espanso_match::event::Key {
  fn from(key: Key) -> Self {
    match key {
      Key::Alt => espanso_match::event::Key::Alt,
      Key::CapsLock => espanso_match::event::Key::CapsLock,
      Key::Control => espanso_match::event::Key::Control,
      Key::Meta => espanso_match::event::Key::Meta,
      Key::NumLock => espanso_match::event::Key::NumLock,
      Key::Shift => espanso_match::event::Key::Shift,
      Key::Enter => espanso_match::event::Key::Enter,
      Key::Tab => espanso_match::event::Key::Tab,
      Key::Space => espanso_match::event::Key::Space,
      Key::ArrowDown => espanso_match::event::Key::ArrowDown,
      Key::ArrowLeft => espanso_match::event::Key::ArrowLeft,
      Key::ArrowRight => espanso_match::event::Key::ArrowRight,
      Key::ArrowUp => espanso_match::event::Key::ArrowUp,
      Key::End => espanso_match::event::Key::End,
      Key::Home => espanso_match::event::Key::Home,
      Key::PageDown => espanso_match::event::Key::PageDown,
      Key::PageUp => espanso_match::event::Key::PageUp,
      Key::Escape => espanso_match::event::Key::Escape,
      Key::Backspace => espanso_match::event::Key::Backspace,
      Key::F1 => espanso_match::event::Key::F1,
      Key::F2 => espanso_match::event::Key::F2,
      Key::F3 => espanso_match::event::Key::F3,
      Key::F4 => espanso_match::event::Key::F4,
      Key::F5 => espanso_match::event::Key::F5,
      Key::F6 => espanso_match::event::Key::F6,
      Key::F7 => espanso_match::event::Key::F7,
      Key::F8 => espanso_match::event::Key::F8,
      Key::F9 => espanso_match::event::Key::F9,
      Key::F10 => espanso_match::event::Key::F10,
      Key::F11 => espanso_match::event::Key::F11,
      Key::F12 => espanso_match::event::Key::F12,
      Key::F13 => espanso_match::event::Key::F13,
      Key::F14 => espanso_match::event::Key::F14,
      Key::F15 => espanso_match::event::Key::F15,
      Key::F16 => espanso_match::event::Key::F16,
      Key::F17 => espanso_match::event::Key::F17,
      Key::F18 => espanso_match::event::Key::F18,
      Key::F19 => espanso_match::event::Key::F19,
      Key::F20 => espanso_match::event::Key::F20,
      Key::Other(_) => espanso_match::event::Key::Other,
    }
  }
}
