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

#[cfg(test)]
pub(crate) mod tests {
  use crate::{
    event::{Event, Key},
    MatchResult, Matcher,
  };

  pub(crate) fn get_matches_after_str<'a, Id: Clone, S, M: Matcher<'a, S, Id>>(
    string: &str,
    matcher: &'a M,
  ) -> Vec<MatchResult<Id>> {
    let mut prev_state = None;
    let mut matches = Vec::new();

    for c in string.chars() {
      let (state, vec_matches) = matcher.process(
        prev_state.as_ref(),
        Event::Key {
          key: Key::Other,
          chars: Some(c.to_string()),
        },
      );

      prev_state = Some(state);
      matches = vec_matches;
    }

    matches
  }
}
