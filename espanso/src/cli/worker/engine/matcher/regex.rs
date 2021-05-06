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

use crate::engine::process::{MatchResult, Matcher, MatcherEvent};
use espanso_match::regex::{RegexMatch, RegexMatcher, RegexMatcherOptions};

use super::MatcherState;

pub struct RegexMatcherAdapterOptions {
  pub max_buffer_size: usize,
}

pub struct RegexMatcherAdapter {
  matcher: RegexMatcher<i32>,
}

impl RegexMatcherAdapter {
  pub fn new(matches: &[RegexMatch<i32>], options: &RegexMatcherAdapterOptions) -> Self {
    let matcher = RegexMatcher::new(matches, RegexMatcherOptions {
      max_buffer_size: options.max_buffer_size,
    });

    Self { matcher }
  }
}

impl<'a> Matcher<'a, MatcherState<'a>> for RegexMatcherAdapter {
  fn process(
    &'a self,
    prev_state: Option<&MatcherState<'a>>,
    event: &MatcherEvent,
  ) -> (MatcherState<'a>, Vec<MatchResult>) {
    use espanso_match::Matcher;

    let prev_state = prev_state.map(|state| {
      if let Some(state) = state.as_regex() {
        state
      } else {
        panic!("invalid state type received in RegexMatcherAdapter")
      }
    });
    let event = event.into();

    let (state, results) = self.matcher.process(prev_state, event);

    let enum_state = MatcherState::Regex(state);
    let results: Vec<MatchResult> = results.into_iter().map(|result| result.into()).collect();

    (enum_state, results)
  }
}
