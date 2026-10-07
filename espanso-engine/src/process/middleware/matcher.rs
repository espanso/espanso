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
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
};

use super::super::Middleware;
use crate::event::{
    input::{Key, Status},
    internal::{DetectedMatch, MatchesDetectedEvent},
    Event, EventType,
};

pub trait Matcher<'a, State> {
    fn process(
        &'a self,
        prev_state: Option<&State>,
        event: &MatcherEvent,
    ) -> (State, Vec<MatchResult>);
}

#[derive(Debug)]
pub enum MatcherEvent {
    Key { key: Key, chars: Option<String> },
    VirtualSeparator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub id: i32,
    pub trigger: String,
    pub left_separator: Option<String>,
    pub right_separator: Option<String>,
    pub args: HashMap<String, String>,
}

pub trait MatcherMiddlewareConfigProvider {
    fn max_history_size(&self) -> usize;
}

pub trait ModifierStateProvider {
    fn get_modifier_state(&self) -> ModifierState;
}

#[derive(Debug, Clone)]
pub struct ModifierState {
    pub is_ctrl_down: bool,
    pub is_alt_down: bool,
    pub is_meta_down: bool,
}

pub struct MatcherMiddleware<'a, State> {
    matchers: &'a [&'a dyn Matcher<'a, State>],

    matcher_states: RefCell<VecDeque<Vec<State>>>,

    max_history_size: usize,

    modifier_status_provider: &'a dyn ModifierStateProvider,

    // True right after an expansion was injected, until the next key event
    after_injection: Cell<bool>,
}

impl<'a, State> MatcherMiddleware<'a, State> {
    pub fn new(
        matchers: &'a [&'a dyn Matcher<'a, State>],
        options_provider: &'a dyn MatcherMiddlewareConfigProvider,
        modifier_status_provider: &'a dyn ModifierStateProvider,
    ) -> Self {
        let max_history_size = options_provider.max_history_size();

        Self {
            matchers,
            matcher_states: RefCell::new(VecDeque::new()),
            max_history_size,
            modifier_status_provider,
            after_injection: Cell::new(false),
        }
    }
}

impl<State> Middleware for MatcherMiddleware<'_, State> {
    fn name(&self) -> &'static str {
        "matcher"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        if is_event_of_interest(&event.etype) {
            let mut matcher_states = self.matcher_states.borrow_mut();
            let prev_states = if matcher_states.is_empty() {
                None
            } else {
                matcher_states.back()
            };

            if let EventType::Keyboard(keyboard_event) = &event.etype {
                // Backspace handling
                if keyboard_event.key == Key::Backspace {
                    let after_injection = self.after_injection.replace(false);
                    let modifiers = self.modifier_status_provider.get_modifier_state();
                    if modifiers.is_alt_down || modifiers.is_ctrl_down || modifiers.is_meta_down {
                        // Option/Ctrl+Backspace delete a whole word, Cmd+Backspace a line: the
                        // buffer no longer reflects the text, popping one state would be wrong
                        trace!("backspace with modifier, clearing matching state");
                        matcher_states.clear();
                    } else if after_injection {
                        // Right after an expansion the last state is the virtual separator, and
                        // the one before it belongs to the typed trigger, not to the text on screen.
                        // Popping back into the trigger broke the next word trigger (#1643).
                        trace!("backspace right after an expansion, clearing matching state");
                        matcher_states.clear();
                    } else {
                        trace!("popping the last matcher state");
                        matcher_states.pop_back();
                    }
                    return event;
                }

                // Keyboard events generated while some modifier keys are pressed are
                // shortcuts, not typed text, so they must not end up in the buffer.
                // See: https://github.com/espanso/espanso/issues/725
                //
                // A shortcut can also change the text without espanso seeing it (a
                // Cmd+V/Super+V paste, for example), so the buffer can no longer be
                // trusted and has to be invalidated as well. Otherwise a match could
                // span the shortcut and its replacement would delete the wrong text.
                // See: https://github.com/espanso/espanso/issues/1851
                if should_skip_key_event_due_to_modifier_press(
                    &self.modifier_status_provider.get_modifier_state(),
                ) {
                    trace!("incompatible modifiers are pressed, clearing matching state");
                    matcher_states.clear();
                    return event;
                }
            }

            // Some keys (such as the arrow keys) and mouse clicks prevent espanso from building
            // an accurate key buffer, so we need to invalidate it.
            if is_invalidating_event(&event.etype) {
                trace!("invalidating event detected, clearing matching state");
                matcher_states.clear();
                return event;
            }

            let mut all_results = Vec::new();

            self.after_injection
                .set(matches!(event.etype, EventType::MatchInjected));

            if let Some(matcher_event) = convert_to_matcher_event(&event.etype) {
                let mut new_states = Vec::new();
                for (i, matcher) in self.matchers.iter().enumerate() {
                    let prev_state = prev_states.and_then(|states| states.get(i));

                    let (state, results) = matcher.process(prev_state, &matcher_event);
                    all_results.extend(results);

                    new_states.push(state);
                }

                matcher_states.push_back(new_states);
                if matcher_states.len() > self.max_history_size {
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
                            is_search: false,
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
        EventType::Keyboard(keyboard_event) if keyboard_event.status == Status::Pressed => {
            // Skip linux Keyboard (XKB) Extension function and modifier keys
            // In hex, they have the byte 3 = 0xfe
            // See list in "keysymdef.h" file
            if cfg!(target_os = "linux") {
                if let (Key::Other(raw_code), None) = (&keyboard_event.key, &keyboard_event.value) {
                    if (65025..=65276).contains(raw_code) {
                        return false;
                    }
                }
            }

            // Skip modifier keys
            !matches!(
                keyboard_event.key,
                Key::Alt | Key::Shift | Key::CapsLock | Key::Meta | Key::NumLock | Key::Control
            )
        }
        EventType::Keyboard(_) => false,
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

fn is_invalidating_event(event_type: &EventType) -> bool {
    match event_type {
        EventType::Keyboard(keyboard_event) => matches!(
            keyboard_event.key,
            Key::ArrowDown
                | Key::ArrowLeft
                | Key::ArrowRight
                | Key::ArrowUp
                | Key::End
                | Key::Home
                | Key::PageDown
                | Key::PageUp
                | Key::Escape
        ),
        EventType::Mouse(_) => true,
        _ => false,
    }
}

fn should_skip_key_event_due_to_modifier_press(modifier_state: &ModifierState) -> bool {
    if cfg!(target_os = "macos") {
        modifier_state.is_meta_down
    } else if cfg!(target_os = "windows") {
        false
    } else if cfg!(target_os = "linux") {
        modifier_state.is_alt_down || modifier_state.is_meta_down
    } else {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::input::KeyboardEvent;
    use std::cell::Cell;

    // Mimics a regex matcher: the state is the typed buffer, and it matches
    // whenever the buffer ends with two spaces.
    struct DoubleSpaceMatcher;

    impl<'a> Matcher<'a, String> for DoubleSpaceMatcher {
        fn process(
            &'a self,
            prev_state: Option<&String>,
            event: &MatcherEvent,
        ) -> (String, Vec<MatchResult>) {
            let mut buffer = prev_state.cloned().unwrap_or_default();
            if let MatcherEvent::Key {
                chars: Some(chars), ..
            } = event
            {
                buffer.push_str(chars);
            }

            let results = if buffer.ends_with("  ") {
                vec![MatchResult {
                    id: 1,
                    trigger: buffer.clone(),
                    left_separator: None,
                    right_separator: None,
                    args: HashMap::new(),
                }]
            } else {
                vec![]
            };

            (buffer, results)
        }
    }

    struct MockConfig;

    impl MatcherMiddlewareConfigProvider for MockConfig {
        fn max_history_size(&self) -> usize {
            3
        }
    }

    #[derive(Default)]
    struct MockModifiers {
        is_meta_down: Cell<bool>,
        is_alt_down: Cell<bool>,
    }

    impl ModifierStateProvider for MockModifiers {
        fn get_modifier_state(&self) -> ModifierState {
            ModifierState {
                is_ctrl_down: false,
                is_alt_down: self.is_alt_down.get(),
                is_meta_down: self.is_meta_down.get(),
            }
        }
    }

    fn key_press(key: Key, value: &str) -> Event {
        Event::caused_by(
            0,
            EventType::Keyboard(KeyboardEvent {
                key,
                value: Some(value.to_string()),
                status: Status::Pressed,
                variant: None,
            }),
        )
    }

    fn detects_match(middleware: &MatcherMiddleware<String>, event: Event) -> bool {
        let result = middleware.next(event, &mut |_| {});
        matches!(result.etype, EventType::MatchesDetected(_))
    }

    #[test]
    fn test_typed_trigger_is_detected() {
        let matcher = DoubleSpaceMatcher;
        let matchers: [&dyn Matcher<String>; 1] = [&matcher];
        let modifiers = MockModifiers::default();
        let middleware = MatcherMiddleware::new(&matchers, &MockConfig, &modifiers);

        assert!(!detects_match(&middleware, key_press(Key::Other(30), "a")));
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
        assert!(detects_match(&middleware, key_press(Key::Space, " ")));
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn test_modifier_shortcut_invalidates_the_buffer() {
        let matcher = DoubleSpaceMatcher;
        let matchers: [&dyn Matcher<String>; 1] = [&matcher];
        let modifiers = MockModifiers::default();
        let middleware = MatcherMiddleware::new(&matchers, &MockConfig, &modifiers);

        assert!(!detects_match(&middleware, key_press(Key::Other(30), "a")));
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));

        // Meta+V pastes text that espanso never sees
        modifiers.is_meta_down.set(true);
        assert!(!detects_match(&middleware, key_press(Key::Other(47), "v")));
        modifiers.is_meta_down.set(false);

        // The space typed after the paste must not complete a match with the
        // space typed before it
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
    }

    fn backspace() -> Event {
        Event::caused_by(
            0,
            EventType::Keyboard(KeyboardEvent {
                key: Key::Backspace,
                value: None,
                status: Status::Pressed,
                variant: None,
            }),
        )
    }

    #[test]
    fn test_backspace_after_expansion_does_not_restore_the_trigger_buffer() {
        // #1643: popping one state after an expansion went back into the typed trigger
        let matcher = DoubleSpaceMatcher;
        let matchers: [&dyn Matcher<String>; 1] = [&matcher];
        let modifiers = MockModifiers::default();
        let middleware = MatcherMiddleware::new(&matchers, &MockConfig, &modifiers);
        assert!(!detects_match(&middleware, key_press(Key::Other(30), "a")));
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
        assert!(detects_match(&middleware, key_press(Key::Space, " ")));
        middleware.next(Event::caused_by(0, EventType::MatchInjected), &mut |_| {});
        middleware.next(backspace(), &mut |_| {});
        assert!(middleware.matcher_states.borrow().is_empty());
        // the buffer before the expansion ("a  ") must not come back
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
    }

    #[test]
    fn test_plain_backspace_still_pops_one_state() {
        let matcher = DoubleSpaceMatcher;
        let matchers: [&dyn Matcher<String>; 1] = [&matcher];
        let modifiers = MockModifiers::default();
        let middleware = MatcherMiddleware::new(&matchers, &MockConfig, &modifiers);
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
        assert!(!detects_match(&middleware, key_press(Key::Other(30), "a")));
        middleware.next(backspace(), &mut |_| {});
        // " " is left, a second space completes the match
        assert!(detects_match(&middleware, key_press(Key::Space, " ")));
    }

    #[test]
    fn test_option_backspace_clears_the_buffer() {
        // Option/Ctrl+Backspace delete a whole word, so one popped state is not enough
        let matcher = DoubleSpaceMatcher;
        let matchers: [&dyn Matcher<String>; 1] = [&matcher];
        let modifiers = MockModifiers::default();
        let middleware = MatcherMiddleware::new(&matchers, &MockConfig, &modifiers);
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
        assert!(!detects_match(&middleware, key_press(Key::Other(30), "a")));
        modifiers.is_alt_down.set(true);
        middleware.next(backspace(), &mut |_| {});
        modifiers.is_alt_down.set(false);
        assert!(middleware.matcher_states.borrow().is_empty());
        assert!(!detects_match(&middleware, key_press(Key::Space, " ")));
    }
}
