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

use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use super::super::Middleware;
use super::matcher::ModifierStateProvider;
use crate::event::{
    input::{Key, Status},
    internal::{TextFormat, UndoEvent},
    Event, EventType,
};

/// Two presses count as a double backspace only if they are this close together.
const DOUBLE_BACKSPACE_WINDOW: Duration = Duration::from_millis(400);

/// X11 key auto-repeat emits release/press pairs with (almost) no gap in between;
/// a real second press is always slower than that.
const MIN_RELEASE_TO_PRESS_GAP: Duration = Duration::from_millis(15);

pub trait UndoEnabledProvider {
    fn is_undo_enabled(&self) -> bool;

    /// How many quick Backspace presses revert an expansion: 1 (default) or 2.
    /// With 2, a single Backspace just deletes the last character as usual.
    fn undo_backspace_presses(&self) -> usize {
        1
    }
}

pub struct UndoMiddleware<'a> {
    undo_enabled_provider: &'a dyn UndoEnabledProvider,
    modifier_state_provider: &'a dyn ModifierStateProvider,
    record: RefCell<Option<InjectionRecord>>,
}

impl<'a> UndoMiddleware<'a> {
    pub fn new(
        undo_enabled_provider: &'a dyn UndoEnabledProvider,
        modifier_state_provider: &'a dyn ModifierStateProvider,
    ) -> Self {
        Self {
            undo_enabled_provider,
            modifier_state_provider,
            record: RefCell::new(None),
        }
    }
}

impl Middleware for UndoMiddleware<'_> {
    fn name(&self) -> &'static str {
        "undo"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        let mut record = self.record.borrow_mut();

        if let EventType::TriggerCompensation(m_event) = &event.etype {
            // The left separator of a word trigger stays on screen when expanding,
            // so it must not be typed again when reverting (#1081)
            let trigger = match &m_event.left_separator {
                Some(separator) => m_event
                    .trigger
                    .strip_prefix(separator.as_str())
                    .unwrap_or(&m_event.trigger)
                    .to_string(),
                None => m_event.trigger.clone(),
            };
            *record = Some(InjectionRecord {
                id: Some(event.source_id),
                trigger: Some(trigger),
                ..Default::default()
            });
        } else if let EventType::Rendered(m_event) = &event.etype {
            if m_event.format == TextFormat::Plain {
                if let Some(record) = &mut *record {
                    if record.id == Some(event.source_id) {
                        record.injected_text = Some(m_event.body.clone());
                        record.match_id = Some(m_event.match_id);
                    }
                }
            }
        } else if let EventType::Keyboard(m_event) = &event.etype {
            if m_event.key == Key::Backspace && m_event.status == Status::Released {
                if let Some(record) = &mut *record {
                    record.released_at = Some(Instant::now());
                }
                return event;
            }

            if m_event.status == Status::Pressed {
                if m_event.key == Key::Backspace
                    && !is_word_or_line_delete(self.modifier_state_provider)
                {
                    if let Some(undo) = self.on_backspace(&mut record) {
                        return Event::caused_by(event.source_id, EventType::Undo(undo));
                    }
                    return event;
                }
                // Any other key (and Option/Ctrl/Cmd+Backspace, which delete more than one
                // character) makes the expansion no longer revertible
                *record = None;
            }
        } else if let EventType::Mouse(_) | EventType::CursorHintCompensation(_) = &event.etype {
            // Explanation:
            // * Any mouse event invalidates the undo feature, as it could
            //   represent a change in application
            // * Cursor hints invalidate the undo feature, as it would be pretty
            //   complex to determine which delete operations should be performed.
            //   This might change in the future.
            *record = None;
        }

        event
    }
}

impl UndoMiddleware<'_> {
    /// Returns the undo request if this Backspace press reverts the expansion.
    fn on_backspace(&self, record: &mut Option<InjectionRecord>) -> Option<UndoEvent> {
        let current = record.as_mut()?;
        if current.trigger.is_none()
            || current.injected_text.is_none()
            || current.match_id.is_none()
        {
            *record = None;
            return None;
        }
        if !self.undo_enabled_provider.is_undo_enabled() {
            *record = None;
            return None;
        }

        let now = Instant::now();
        let presses_needed = self
            .undo_enabled_provider
            .undo_backspace_presses()
            .clamp(1, 2);
        if presses_needed == 2 {
            match current.first_backspace_at {
                None => {
                    // First press: deletes one character as usual, wait for a second one
                    current.first_backspace_at = Some(now);
                    current.released_at = None;
                    return None;
                }
                Some(first) => {
                    let is_quick = now.duration_since(first) <= DOUBLE_BACKSPACE_WINDOW;
                    // A held key repeats without a release (macOS) or with a release
                    // immediately followed by the next press (X11): both are not a double press
                    let was_released = current.released_at.is_some_and(|released| {
                        now.duration_since(released) >= MIN_RELEASE_TO_PRESS_GAP
                    });
                    if !(is_quick && was_released) {
                        *record = None;
                        return None;
                    }
                }
            }
        }

        let current = record.take()?;
        Some(UndoEvent {
            match_id: current.match_id?,
            trigger: current.trigger?,
            replace: current.injected_text?,
            deleted_chars: presses_needed,
        })
    }
}

fn is_word_or_line_delete(modifier_state_provider: &dyn ModifierStateProvider) -> bool {
    let state = modifier_state_provider.get_modifier_state();
    state.is_alt_down || state.is_ctrl_down || state.is_meta_down
}

#[derive(Default)]
struct InjectionRecord {
    id: Option<u32>,
    match_id: Option<i32>,
    trigger: Option<String>,
    injected_text: Option<String>,
    first_backspace_at: Option<Instant>,
    released_at: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        effect::TriggerCompensationEvent, input::KeyboardEvent, internal::RenderedEvent,
    };
    use crate::process::middleware::matcher::ModifierState;
    use std::cell::Cell;

    struct Config {
        presses: usize,
    }
    impl UndoEnabledProvider for Config {
        fn is_undo_enabled(&self) -> bool {
            true
        }
        fn undo_backspace_presses(&self) -> usize {
            self.presses
        }
    }

    #[derive(Default)]
    struct Modifiers {
        alt: Cell<bool>,
    }
    impl ModifierStateProvider for Modifiers {
        fn get_modifier_state(&self) -> ModifierState {
            ModifierState {
                is_ctrl_down: false,
                is_alt_down: self.alt.get(),
                is_meta_down: false,
            }
        }
    }

    fn expand(middleware: &UndoMiddleware, left_separator: Option<&str>) {
        let trigger = format!("{}ggr", left_separator.unwrap_or(""));
        middleware.next(
            Event::caused_by(
                7,
                EventType::TriggerCompensation(TriggerCompensationEvent {
                    trigger,
                    left_separator: left_separator.map(str::to_string),
                }),
            ),
            &mut |_| {},
        );
        middleware.next(
            Event::caused_by(
                7,
                EventType::Rendered(RenderedEvent {
                    match_id: 3,
                    body: "geringgradig".to_string(),
                    format: TextFormat::Plain,
                }),
            ),
            &mut |_| {},
        );
    }

    fn key(key: Key, status: Status) -> Event {
        Event::caused_by(
            8,
            EventType::Keyboard(KeyboardEvent {
                key,
                value: None,
                status,
                variant: None,
            }),
        )
    }

    fn press_backspace(middleware: &UndoMiddleware) -> Option<UndoEvent> {
        match middleware
            .next(key(Key::Backspace, Status::Pressed), &mut |_| {})
            .etype
        {
            EventType::Undo(undo) => Some(undo),
            _ => None,
        }
    }

    fn release_backspace(middleware: &UndoMiddleware) {
        middleware.next(key(Key::Backspace, Status::Released), &mut |_| {});
    }

    fn wait(ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
    }

    #[test]
    fn single_press_reverts_like_before() {
        let (config, modifiers) = (Config { presses: 1 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        let undo = press_backspace(&middleware).expect("undo");
        assert_eq!(undo.trigger, "ggr");
        assert_eq!(undo.replace, "geringgradig");
        assert_eq!(undo.deleted_chars, 1);
    }

    #[test]
    fn left_separator_is_not_typed_again() {
        // #1081: undoing a word match inserted an extra space
        let (config, modifiers) = (Config { presses: 1 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, Some(" "));
        assert_eq!(press_backspace(&middleware).expect("undo").trigger, "ggr");
    }

    #[test]
    fn double_press_reverts_on_the_second_press() {
        let (config, modifiers) = (Config { presses: 2 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        assert!(
            press_backspace(&middleware).is_none(),
            "first press only deletes one char"
        );
        wait(30);
        release_backspace(&middleware);
        wait(30);
        let undo = press_backspace(&middleware).expect("second quick press reverts");
        assert_eq!(undo.deleted_chars, 2);
    }

    #[test]
    fn slow_second_press_does_not_revert() {
        let (config, modifiers) = (Config { presses: 2 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        assert!(press_backspace(&middleware).is_none());
        release_backspace(&middleware);
        wait(450);
        assert!(press_backspace(&middleware).is_none());
        wait(30);
        release_backspace(&middleware);
        assert!(
            press_backspace(&middleware).is_none(),
            "record is gone after a slow press"
        );
    }

    #[test]
    fn held_backspace_does_not_revert() {
        // auto-repeat: presses without a release in between
        let (config, modifiers) = (Config { presses: 2 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        assert!(press_backspace(&middleware).is_none());
        wait(40);
        assert!(press_backspace(&middleware).is_none());
    }

    #[test]
    fn x11_style_repeat_does_not_revert() {
        // X11 auto-repeat sends release + press with no gap
        let (config, modifiers) = (Config { presses: 2 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        assert!(press_backspace(&middleware).is_none());
        wait(40);
        release_backspace(&middleware);
        assert!(press_backspace(&middleware).is_none());
    }

    #[test]
    fn option_backspace_never_reverts() {
        let (config, modifiers) = (Config { presses: 1 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        modifiers.alt.set(true);
        assert!(press_backspace(&middleware).is_none());
        modifiers.alt.set(false);
        assert!(press_backspace(&middleware).is_none(), "record discarded");
    }

    #[test]
    fn typing_in_between_cancels() {
        let (config, modifiers) = (Config { presses: 2 }, Modifiers::default());
        let middleware = UndoMiddleware::new(&config, &modifiers);
        expand(&middleware, None);
        assert!(press_backspace(&middleware).is_none());
        release_backspace(&middleware);
        middleware.next(key(Key::Other(30), Status::Pressed), &mut |_| {});
        wait(20);
        assert!(press_backspace(&middleware).is_none());
    }
}
