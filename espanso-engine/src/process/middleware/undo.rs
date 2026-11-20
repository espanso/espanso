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

use std::cell::RefCell;

use super::super::Middleware;
use super::matcher::ModifierStateProvider;
use crate::event::{
    input::{Key, Status},
    internal::{TextFormat, UndoEvent},
    Event, EventType,
};

pub trait UndoEnabledProvider {
    fn is_undo_enabled(&self) -> bool;
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
            *record = Some(InjectionRecord {
                id: Some(event.source_id),
                trigger: Some(m_event.trigger.clone()),
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
            if m_event.status == Status::Pressed {
                // A word or line delete has already removed part of the injected text, and
                // undoing would then delete more than the expansion, so only a plain
                // Backspace undoes. Any other key, this one included, drops the record.
                if m_event.key == Key::Backspace
                    && !self
                        .modifier_state_provider
                        .get_modifier_state()
                        .is_word_or_line_delete()
                {
                    if let Some(record) = (*record).take() {
                        if let (Some(trigger), Some(injected_text), Some(match_id)) =
                            (record.trigger, record.injected_text, record.match_id)
                        {
                            if self.undo_enabled_provider.is_undo_enabled() {
                                return Event::caused_by(
                                    event.source_id,
                                    EventType::Undo(UndoEvent {
                                        match_id,
                                        trigger,
                                        replace: injected_text,
                                    }),
                                );
                            }
                        }
                    }
                }
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

#[derive(Default)]
struct InjectionRecord {
    id: Option<u32>,
    match_id: Option<i32>,
    trigger: Option<String>,
    injected_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        effect::TriggerCompensationEvent, input::KeyboardEvent, internal::RenderedEvent,
    };
    use crate::process::ModifierState;
    use std::cell::Cell;

    struct UndoEnabled;

    impl UndoEnabledProvider for UndoEnabled {
        fn is_undo_enabled(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct MockModifiers {
        is_ctrl_down: Cell<bool>,
        is_alt_down: Cell<bool>,
        is_meta_down: Cell<bool>,
    }

    impl ModifierStateProvider for MockModifiers {
        fn get_modifier_state(&self) -> ModifierState {
            ModifierState {
                is_ctrl_down: self.is_ctrl_down.get(),
                is_alt_down: self.is_alt_down.get(),
                is_meta_down: self.is_meta_down.get(),
            }
        }
    }

    // Feeds the events of an expansion of "btw " into "by the way "
    fn expand(middleware: &UndoMiddleware) {
        let source_id = 1;
        middleware.next(
            Event::caused_by(
                source_id,
                EventType::TriggerCompensation(TriggerCompensationEvent {
                    trigger: "btw ".to_string(),
                    left_separator: None,
                }),
            ),
            &mut |_| {},
        );
        middleware.next(
            Event::caused_by(
                source_id,
                EventType::Rendered(RenderedEvent {
                    match_id: 7,
                    body: "by the way ".to_string(),
                    format: TextFormat::Plain,
                }),
            ),
            &mut |_| {},
        );
    }

    fn backspace(middleware: &UndoMiddleware) -> Event {
        middleware.next(
            Event::caused_by(
                2,
                EventType::Keyboard(KeyboardEvent {
                    key: Key::Backspace,
                    value: None,
                    status: Status::Pressed,
                    variant: None,
                }),
            ),
            &mut |_| {},
        )
    }

    fn key_press(middleware: &UndoMiddleware, key: Key) -> Event {
        middleware.next(
            Event::caused_by(
                2,
                EventType::Keyboard(KeyboardEvent {
                    key,
                    value: None,
                    status: Status::Pressed,
                    variant: None,
                }),
            ),
            &mut |_| {},
        )
    }

    fn is_undo(event: &Event) -> bool {
        matches!(event.etype, EventType::Undo(_))
    }

    #[test]
    fn test_plain_backspace_after_expansion_undoes() {
        let modifiers = MockModifiers::default();
        let middleware = UndoMiddleware::new(&UndoEnabled, &modifiers);
        expand(&middleware);
        let event = backspace(&middleware);
        let EventType::Undo(undo) = &event.etype else {
            panic!("expected an undo event, got {:?}", event.etype);
        };
        assert_eq!(undo.trigger, "btw ");
        assert_eq!(undo.replace, "by the way ");
        assert_eq!(undo.match_id, 7);
    }

    #[test]
    fn test_word_or_line_delete_after_expansion_does_not_undo() {
        // The app has already deleted the last word or line of the expansion; undoing would
        // delete the full expansion length on top and eat text typed before it.
        let setters: [fn(&MockModifiers, bool); 3] = [
            |m, v| m.is_alt_down.set(v),
            |m, v| m.is_ctrl_down.set(v),
            |m, v| m.is_meta_down.set(v),
        ];
        for set in setters {
            let modifiers = MockModifiers::default();
            let middleware = UndoMiddleware::new(&UndoEnabled, &modifiers);
            expand(&middleware);
            set(&modifiers, true);
            assert!(!is_undo(&backspace(&middleware)));
            set(&modifiers, false);
            // the record is gone, a later plain Backspace must not undo either
            assert!(!is_undo(&backspace(&middleware)));
        }
    }

    #[test]
    fn test_modifier_press_before_backspace_drops_the_record() {
        // The usual sequence: the modifier press arrives as its own key event and already drops
        // the record, so the modifier check above only matters when that press was missed
        // (e.g. discarded during the injection, or swallowed by the OS).
        let modifiers = MockModifiers::default();
        let middleware = UndoMiddleware::new(&UndoEnabled, &modifiers);
        expand(&middleware);
        key_press(&middleware, Key::Alt);
        modifiers.is_alt_down.set(true);
        assert!(!is_undo(&backspace(&middleware)));
    }
}
