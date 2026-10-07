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
    record: RefCell<Option<InjectionRecord>>,
}

impl<'a> UndoMiddleware<'a> {
    pub fn new(undo_enabled_provider: &'a dyn UndoEnabledProvider) -> Self {
        Self {
            undo_enabled_provider,
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
            if m_event.status == Status::Pressed {
                if m_event.key == Key::Backspace {
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

    struct Enabled;
    impl UndoEnabledProvider for Enabled {
        fn is_undo_enabled(&self) -> bool {
            true
        }
    }

    fn undo_after_expansion(left_separator: Option<&str>) -> Option<UndoEvent> {
        let middleware = UndoMiddleware::new(&Enabled);
        middleware.next(
            Event::caused_by(
                1,
                EventType::TriggerCompensation(TriggerCompensationEvent {
                    trigger: format!("{}brb", left_separator.unwrap_or("")),
                    left_separator: left_separator.map(str::to_string),
                }),
            ),
            &mut |_| {},
        );
        middleware.next(
            Event::caused_by(
                1,
                EventType::Rendered(RenderedEvent {
                    match_id: 7,
                    body: "be right back".to_string(),
                    format: TextFormat::Plain,
                }),
            ),
            &mut |_| {},
        );
        let backspace = Event::caused_by(
            2,
            EventType::Keyboard(KeyboardEvent {
                key: Key::Backspace,
                value: None,
                status: Status::Pressed,
                variant: None,
            }),
        );
        match middleware.next(backspace, &mut |_| {}).etype {
            EventType::Undo(undo) => Some(undo),
            _ => None,
        }
    }

    #[test]
    fn undo_retypes_the_plain_trigger() {
        let undo = undo_after_expansion(None).expect("undo");
        assert_eq!(undo.trigger, "brb");
        assert_eq!(undo.replace, "be right back");
    }

    #[test]
    fn undo_does_not_retype_the_left_separator() {
        // #1081: the separator before a word trigger is still on screen
        assert_eq!(
            undo_after_expansion(Some(" ")).expect("undo").trigger,
            "brb"
        );
    }
}
