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

use crate::config::ConfigManager;
use crate::event::KeyModifier::{BACKSPACE, CAPS_LOCK, LEFT_SHIFT, RIGHT_SHIFT};
use crate::event::{ActionEventReceiver, ActionType, KeyModifier};
use crate::matcher::{Match, MatchReceiver, TriggerEntry};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::SystemTime;

pub struct ScrollingMatcher<'a, R: MatchReceiver, M: ConfigManager<'a>> {
    config_manager: &'a M,
    receiver: &'a R,
    current_set_queue: RefCell<VecDeque<Vec<MatchEntry<'a>>>>,
    toggle_press_time: RefCell<SystemTime>,
    passive_press_time: RefCell<SystemTime>,
    is_enabled: RefCell<bool>,
    was_previous_char_word_separator: RefCell<bool>,
    was_previous_char_a_match: RefCell<bool>,
}

#[derive(Clone)]
struct MatchEntry<'a> {
    start: usize,
    count: usize,
    trigger_offset: usize, // The index of the trigger in the Match that matched
    _match: &'a Match,
}

impl<'a, R: MatchReceiver, M: ConfigManager<'a>> ScrollingMatcher<'a, R, M> {
    pub fn new(config_manager: &'a M, receiver: &'a R) -> ScrollingMatcher<'a, R, M> {
        let current_set_queue = RefCell::new(VecDeque::new());
        let toggle_press_time = RefCell::new(SystemTime::now());
        let passive_press_time = RefCell::new(SystemTime::now());

        ScrollingMatcher {
            config_manager,
            receiver,
            current_set_queue,
            toggle_press_time,
            passive_press_time,
            is_enabled: RefCell::new(true),
            was_previous_char_word_separator: RefCell::new(true),
            was_previous_char_a_match: RefCell::new(true),
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

    fn is_matching(
        mtc: &Match,
        current_char: &str,
        start: usize,
        trigger_offset: usize,
        is_current_word_separator: bool,
    ) -> bool {
        match mtc._trigger_sequences[trigger_offset][start] {
            TriggerEntry::Char(c) => current_char.starts_with(c),
            TriggerEntry::WordSeparator => is_current_word_separator,
        }
    }
}

impl<'a, R: MatchReceiver, M: ConfigManager<'a>> super::Matcher for ScrollingMatcher<'a, R, M> {
    fn handle_char(&self, c: &str) {
        // if not enabled, avoid any processing
        if !*(self.is_enabled.borrow()) {
            return;
        }

        // Obtain the configuration for the active application if present,
        // otherwise get the default one
        let active_config = self.config_manager.active_config();

        // Check if the current char is a word separator
        let mut is_current_word_separator = active_config
            .word_separators
            .contains(&c.chars().nth(0).unwrap_or_default());

        let mut was_previous_char_a_match = self.was_previous_char_a_match.borrow_mut();
        (*was_previous_char_a_match) = false;

        let mut was_previous_word_separator = self.was_previous_char_word_separator.borrow_mut();

        let mut current_set_queue = self.current_set_queue.borrow_mut();

        let mut new_matches: Vec<MatchEntry> = Vec::new();

        for m in active_config.matches.iter() {
            // only active-enabled matches are considered
            if m.passive_only {
                continue;
            }

            for trigger_offset in 0..m._trigger_sequences.len() {
                let mut result =
                    Self::is_matching(m, c, 0, trigger_offset, is_current_word_separator);

                if m.word {
                    result = result && *was_previous_word_separator
                }

                if result {
                    new_matches.push(MatchEntry {
                        start: 1,
                        count: m._trigger_sequences[trigger_offset].len(),
                        trigger_offset,
                        _match: &m,
                    });
                }
            }
        }
        // TODO: use an associative structure to improve the efficiency of this first "new_matches" lookup.

        let combined_matches: Vec<MatchEntry> = match current_set_queue.back_mut() {
            Some(last_matches) => {
                let mut updated: Vec<MatchEntry> = last_matches
                    .iter()
                    .filter(|&x| {
                        Self::is_matching(
                            x._match,
                            c,
                            x.start,
                            x.trigger_offset,
                            is_current_word_separator,
                        )
                    })
                    .map(|x| MatchEntry {
                        start: x.start + 1,
                        count: x.count,
                        trigger_offset: x.trigger_offset,
                        _match: &x._match,
                    })
                    .collect();

                updated.extend(new_matches);
                updated
            }
            None => new_matches,
        };

        let mut found_entry = None;

        for entry in combined_matches.iter() {
            if entry.start == entry.count {
                found_entry = Some(entry.clone());
                break;
            }
        }

        current_set_queue.push_back(combined_matches);

        if current_set_queue.len() as i32
            > (self.config_manager.default_config().backspace_limit + 1)
        {
            current_set_queue.pop_front();
        }

        *was_previous_word_separator = is_current_word_separator;

        if let Some(entry) = found_entry {
            let mtc = entry._match;

            current_set_queue.clear();

            let trailing_separator = if !mtc.word {
                // If it's not a word match, it cannot have a trailing separator
                None
            } else if !is_current_word_separator {
                None
            } else {
                let as_char = c.chars().nth(0);
                match as_char {
                    Some(c) => {
                        Some(c) // Current char is the trailing separator
                    }
                    None => None,
                }
            };

            // Force espanso to consider the last char as a separator
            *was_previous_word_separator = true;

            self.receiver
                .on_match(mtc, trailing_separator, entry.trigger_offset);

            (*was_previous_char_a_match) = true;
        }
    }

    fn handle_modifier(&self, m: KeyModifier) {
        let config = self.config_manager.default_config();

        let mut was_previous_char_a_match = self.was_previous_char_a_match.borrow_mut();

        // TODO: at the moment, activating the passive key triggers the toggle key
        // study a mechanism to avoid this problem

        if KeyModifier::shallow_equals(&m, &config.toggle_key) {
            check_interval(
                &self.toggle_press_time,
                u128::from(config.toggle_interval),
                || {
                    self.toggle();

                    let is_enabled = self.is_enabled.borrow();

                    if !*is_enabled {
                        self.current_set_queue.borrow_mut().clear();
                    }
                },
            );
        } else if KeyModifier::shallow_equals(&m, &config.passive_key) {
            check_interval(
                &self.passive_press_time,
                u128::from(config.toggle_interval),
                || {
                    self.receiver.on_passive();
                },
            );
        }

        // Backspace handling, basically "rewinding history"
        if m == BACKSPACE {
            let mut current_set_queue = self.current_set_queue.borrow_mut();
            current_set_queue.pop_back();

            if (*was_previous_char_a_match) {
                current_set_queue.clear();
                self.receiver.on_undo();
            }
        }

        // Disable the "backspace undo" feature
        (*was_previous_char_a_match) = false;

        // Consider modifiers as separators to improve word matches reliability
        if m != LEFT_SHIFT && m != RIGHT_SHIFT && m != CAPS_LOCK {
            let mut was_previous_char_word_separator =
                self.was_previous_char_word_separator.borrow_mut();
            *was_previous_char_word_separator = true;
        }
    }

    fn handle_other(&self) {
        // When receiving "other" type of events, we mark them as valid separators.
        // This dramatically improves the reliability of word matches
        let mut was_previous_char_word_separator =
            self.was_previous_char_word_separator.borrow_mut();
        *was_previous_char_word_separator = true;

        // Disable the "backspace undo" feature
        let mut was_previous_char_a_match = self.was_previous_char_a_match.borrow_mut();
        (*was_previous_char_a_match) = false;
    }
}

impl<'a, R: MatchReceiver, M: ConfigManager<'a>> ActionEventReceiver
    for ScrollingMatcher<'a, R, M>
{
    fn on_action_event(&self, e: ActionType) {
        match e {
            ActionType::Toggle => {
                self.toggle();
            }
            ActionType::Enable => {
                self.set_enabled(true);
            }
            ActionType::Disable => {
                self.set_enabled(false);
            }
            _ => {}
        }
    }
}

fn check_interval<F>(state_var: &RefCell<SystemTime>, interval: u128, elapsed_callback: F)
where
    F: Fn(),
{
    let mut press_time = state_var.borrow_mut();
    if let Ok(elapsed) = press_time.elapsed() {
        if elapsed.as_millis() < interval {
            elapsed_callback();
        }
    }

    (*press_time) = SystemTime::now();
}
