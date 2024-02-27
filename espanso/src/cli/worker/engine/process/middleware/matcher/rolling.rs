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

use espanso_match::rolling::{
    matcher::{RollingMatcher, RollingMatcherOptions},
    RollingMatch,
};

use espanso_engine::process::{MatchResult, Matcher, MatcherEvent};

use super::{convert_to_engine_result, convert_to_match_event, MatcherState};

pub struct RollingMatcherAdapterOptions {
    pub char_word_separators: Vec<String>,
}

pub struct RollingMatcherAdapter {
    matcher: RollingMatcher<i32>,
}

impl RollingMatcherAdapter {
    pub fn new(matches: &[RollingMatch<i32>], options: RollingMatcherAdapterOptions) -> Self {
        let matcher = RollingMatcher::new(
            matches,
            RollingMatcherOptions {
                char_word_separators: options.char_word_separators,
                key_word_separators: vec![], // TODO?
            },
        );

        Self { matcher }
    }
}

impl<'a> Matcher<'a, MatcherState<'a>> for RollingMatcherAdapter {
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
        let event = convert_to_match_event(event);

        let (state, results) = self.matcher.process(prev_state, event);

        let enum_state = MatcherState::Rolling(state);
        let results: Vec<MatchResult> = results.into_iter().map(convert_to_engine_result).collect();

        (enum_state, results)
    }
}
