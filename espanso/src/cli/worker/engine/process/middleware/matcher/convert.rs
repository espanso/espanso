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

use espanso_config::{
    config::ConfigStore,
    matches::{
        store::{MatchSet, MatchStore},
        MatchCause,
    },
};
use espanso_detect::hotkey::HotKey;
use espanso_match::{
    regex::RegexMatch,
    rolling::{RollingMatch, StringMatchOptions},
};
use log::error;

use crate::cli::worker::builtin::BuiltInMatch;

pub struct MatchConverter<'a> {
    config_store: &'a dyn ConfigStore,
    match_store: &'a dyn MatchStore,
    builtin_matches: &'a [BuiltInMatch],
}

impl<'a> MatchConverter<'a> {
    pub fn new(
        config_store: &'a dyn ConfigStore,
        match_store: &'a dyn MatchStore,
        builtin_matches: &'a [BuiltInMatch],
    ) -> Self {
        Self {
            config_store,
            match_store,
            builtin_matches,
        }
    }

    // TODO: test (might need to move the conversion logic into a separate function)
    pub fn get_rolling_matches(&self) -> Vec<RollingMatch<i32>> {
        let match_set = self.global_match_set();
        let mut matches = Vec::new();

        // First convert configuration (user-defined) matches
        for m in match_set.matches {
            if let MatchCause::Trigger(cause) = &m.cause {
                for trigger in &cause.triggers {
                    matches.push(RollingMatch::from_string(
                        m.id,
                        trigger,
                        &StringMatchOptions {
                            case_insensitive: cause.propagate_case,
                            left_word: cause.left_word,
                            right_word: cause.right_word,
                        },
                    ));
                }
            }
        }

        // Then convert built-in ones
        for m in self.builtin_matches {
            for trigger in &m.triggers {
                matches.push(RollingMatch::from_string(
                    m.id,
                    trigger,
                    &StringMatchOptions::default(),
                ));
            }
        }

        matches
    }

    // TODO: test (might need to move the conversion logic into a separate function)
    pub fn get_regex_matches(&self) -> Vec<RegexMatch<i32>> {
        let match_set = self.global_match_set();
        let mut matches = Vec::new();

        for m in match_set.matches {
            if let MatchCause::Regex(cause) = &m.cause {
                matches.push(RegexMatch::new(m.id, &cause.regex));
            }
        }

        matches
    }

    pub fn get_hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = Vec::new();

        // TODO: read user-defined matches

        // Then convert built-in ones
        for m in self.builtin_matches {
            if let Some(hotkey) = &m.hotkey {
                match HotKey::new(m.id, hotkey) {
                    Ok(hotkey) => hotkeys.push(hotkey),
                    Err(err) => {
                        error!("unable to register hotkey: {}, with error: {}", hotkey, err);
                    }
                }
            }
        }

        hotkeys
    }

    fn global_match_set(&self) -> MatchSet {
        let paths = self.config_store.get_all_match_paths();
        self.match_store
            .query(&paths.into_iter().collect::<Vec<_>>())
    }
}
