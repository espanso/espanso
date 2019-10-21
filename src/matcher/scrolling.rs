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

use crate::matcher::{Match, MatchReceiver};
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
    _match: &'a Match,

    // Usually false, becomes true if the match was detected and has the "word" option.
    // This is needed to trigger the replacement only if the next char is a
    // word separator ( such as space ).
    waiting_for_separator: bool,
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
        let is_current_word_separator = active_config.word_separators.contains(
            &c.chars().nth(0).unwrap_or_default()
        );

        let mut was_previous_word_separator = self.was_previous_char_word_separator.borrow_mut();

        let mut current_set_queue = self.current_set_queue.borrow_mut();

        let new_matches: Vec<MatchEntry> = active_config.matches.iter()
            .filter(|&x| {
                if !x.trigger.starts_with(c) {
                    false
                }else{
                    // If word option is true, a match can only be started if the previous
                    // char was a word separator
                    if x.word {
                        *was_previous_word_separator
                    }else{
                        true
                    }
                }
            })
            .map(|x | MatchEntry{
                start: 1,
                count: x.trigger.chars().count(),
                _match: &x,
                waiting_for_separator: false
            })
            .collect();
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let mut combined_matches: Vec<MatchEntry> = match current_set_queue.back_mut() {
            Some(last_matches) => {
                let mut updated: Vec<MatchEntry> = last_matches.iter()
                    .filter(|&x| {
                        if x.waiting_for_separator {
                            // The match is only waiting for a separator to call the replacement
                            is_current_word_separator
                        }else{
                            let nchar = x._match.trigger.chars().nth(x.start);
                            if let Some(nchar) = nchar {
                                c.starts_with(nchar)
                            }else{
                                false
                            }
                        }
                    })
                    .map(|x | {
                        let new_start = if x.waiting_for_separator {
                            x.start  // Avoid incrementing, we are only waiting for a separator
                        }else{
                            x.start+1  // Increment, we want to check if the next char matches
                        };

                        MatchEntry{
                            start: new_start,
                            count: x.count,
                            _match: &x._match,
                            waiting_for_separator: x.waiting_for_separator
                        }
                    })
                    .collect();

                updated.extend(new_matches);
                updated
            },
            None => {new_matches},
        };

        let mut found_match = None;

        for entry in combined_matches.iter_mut() {
            let is_found_match = if entry.start == entry.count {
                if !entry._match.word {
                    true
                }else{
                    if entry.waiting_for_separator {
                        true
                    }else{
                        entry.waiting_for_separator = true;
                        false
                    }
                }
            }else{
                false
            };

            if is_found_match {
                found_match = Some(entry.clone());
                break;
            }
        }

        current_set_queue.push_back(combined_matches);

        if current_set_queue.len() as i32 > (self.config_manager.default_config().backspace_limit + 1) {
            current_set_queue.pop_front();
        }

        *was_previous_word_separator = is_current_word_separator;

        if let Some(match_entry) = found_match {
            if let Some(last) = current_set_queue.back_mut() {
                last.clear();
            }

            let trailing_separator = if !match_entry.waiting_for_separator {
                None
            }else{
                // Force espanso to consider the previous char a word separator after a match
                *was_previous_word_separator = true;

                let as_char = c.chars().nth(0);
                match as_char {
                    Some(c) => {
                        Some(c) // Current char is the trailing separator
                    },
                    None => {None},
                }
            };

            self.receiver.on_match(match_entry._match, trailing_separator);
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