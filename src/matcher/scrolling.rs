/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use crate::matcher::{Match, MatchReceiver, TriggerEntry};
use std::cell::RefCell;
use crate::event::{KeyModifier, ActionEventReceiver, ActionType};
use crate::config::ConfigManager;
use crate::event::KeyModifier::BACKSPACE;
use std::time::SystemTime;
use std::collections::VecDeque;

pub struct ScrollingMatcher<'a, R: MatchReceiver, M: ConfigManager<'a>> {
    config_manager: &'a M,
    receiver: &'a R,
    current_set_queue: RefCell<VecDeque<Vec<MatchEntry<'a>>>>,
    toggle_press_time: RefCell<SystemTime>,
    is_enabled: RefCell<bool>,
    was_previous_char_word_separator: RefCell<bool>,
}

#[derive(Clone)]
struct MatchEntry<'a> {
    start: usize,
    count: usize,
    _match: &'a Match
}

impl <'a, R: MatchReceiver, M: ConfigManager<'a>> ScrollingMatcher<'a, R, M> {
    pub fn new(config_manager: &'a M, receiver: &'a R) -> ScrollingMatcher<'a, R, M> {
        let current_set_queue = RefCell::new(VecDeque::new());
        let toggle_press_time = RefCell::new(SystemTime::now());

        ScrollingMatcher{
            config_manager,
            receiver,
            current_set_queue,
            toggle_press_time,
            is_enabled: RefCell::new(true),
            was_previous_char_word_separator: RefCell::new(true),
        }
    }

    fn toggle(&self) {
        let mut is_enabled = self.is_enabled.borrow_mut();
        *is_enabled = !(*is_enabled);

        self.receiver.on_enable_update(*is_enabled);
    }

    fn set_enabled(&self, enabled: bool) {
        let mut is_enabled = self.is_enabled.borrow_mut();
        *is_enabled = enabled;

        self.receiver.on_enable_update(*is_enabled);
    }

    fn is_matching(mtc: &Match, current_char: &str, start: usize, is_current_word_separator: bool) -> bool {
        match mtc._trigger_sequence[start] {
            TriggerEntry::Char(c) => {
                current_char.starts_with(c)
            },
            TriggerEntry::WordSeparator => {
                is_current_word_separator
            },
        }
    }
}

impl <'a, R: MatchReceiver, M: ConfigManager<'a>> super::Matcher for ScrollingMatcher<'a, R, M> {
    fn handle_char(&self, c: &str) {
        // if not enabled, avoid any processing
        if !*(self.is_enabled.borrow()) {
            return;
        }

        // Obtain the configuration for the active application if present,
        // otherwise get the default one
        let active_config = self.config_manager.active_config();

        // Check if the current char is a word separator
        let mut is_current_word_separator = active_config.word_separators.contains(
            &c.chars().nth(0).unwrap_or_default()
        );

        // Workaround needed on macos to consider espanso replacement key presses as separators.
        if cfg!(target_os = "macos") {
            if c.len() > 1 {
                is_current_word_separator = true;
            }
        }

        let mut was_previous_word_separator = self.was_previous_char_word_separator.borrow_mut();

        let mut current_set_queue = self.current_set_queue.borrow_mut();

        let new_matches: Vec<MatchEntry> = active_config.matches.iter()
            .filter(|&x| {
                let mut result = Self::is_matching(x, c, 0, is_current_word_separator);

                if x.word {
                    result = result && *was_previous_word_separator
                }

                result
            })
            .map(|x | MatchEntry{
                start: 1,
                count: x._trigger_sequence.len(),
                _match: &x
            })
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let combined_matches: Vec<MatchEntry> = match current_set_queue.back_mut() {
            Some(last_matches) => {
                let mut updated: Vec<MatchEntry> = last_matches.iter()
                    .filter(|&x| {
                        Self::is_matching(x._match, c, x.start, is_current_word_separator)
                    })
                    .map(|x | MatchEntry{
                        start: x.start+1,
                        count: x.count,
                        _match: &x._match
                    })
                    .collect();

                updated.extend(new_matches);
                updated
            },
            None => {new_matches},
        };

        let mut found_match = None;

        for entry in combined_matches.iter() {
            if entry.start == entry.count {
                found_match = Some(entry._match);
                break;
            }
        }

        current_set_queue.push_back(combined_matches);

        if current_set_queue.len() as i32 > (self.config_manager.default_config().backspace_limit + 1) {
            current_set_queue.pop_front();
        }

        *was_previous_word_separator = is_current_word_separator;

        if let Some(mtc) = found_match {
            if let Some(last) = current_set_queue.back_mut() {
                last.clear();
            }

            let trailing_separator = if !is_current_word_separator {
                None
            }else{
                let as_char = c.chars().nth(0);
                match as_char {
                    Some(c) => {
                        Some(c) // Current char is the trailing separator
                    },
                    None => {None},
                }
            };

            // Force espanso to consider the last char as a separator
            *was_previous_word_separator = true;

            self.receiver.on_match(mtc, trailing_separator);
        }
    }

    fn handle_modifier(&self, m: KeyModifier) {
        let config = self.config_manager.default_config();

        if m == config.toggle_key {
            let mut toggle_press_time = self.toggle_press_time.borrow_mut();
            if let Ok(elapsed) = toggle_press_time.elapsed() {
                if elapsed.as_millis() < u128::from(config.toggle_interval) {
                    self.toggle();

                    let is_enabled = self.is_enabled.borrow();

                    if !*is_enabled {
                        self.current_set_queue.borrow_mut().clear();
                    }
                }
            }

            (*toggle_press_time) = SystemTime::now();
        }

        // Backspace handling, basically "rewinding history"
        if m == BACKSPACE {
            let mut current_set_queue = self.current_set_queue.borrow_mut();
            current_set_queue.pop_back();
        }
    }
}

impl <'a, R: MatchReceiver, M: ConfigManager<'a>> ActionEventReceiver for ScrollingMatcher<'a, R, M> {
    fn on_action_event(&self, e: ActionType) {
        match e {
            ActionType::Toggle => {
                self.toggle();
            },
            ActionType::Enable => {
                self.set_enabled(true);
            },
            ActionType::Disable => {
                self.set_enabled(false);
            },
            _ => {}
        }
    }
}