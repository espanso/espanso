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

use crossbeam::channel::{Receiver, Select, SelectedOperation};
use espanso_detect::event::InputEvent;

use espanso_engine::{
  event::{
    input::{HotKeyEvent, Key, KeyboardEvent, MouseButton, MouseEvent, Status, Variant},
    Event, EventType, SourceId,
  },
  funnel,
};

pub struct DetectSource {
  pub receiver: Receiver<(InputEvent, SourceId)>,
}

impl<'a> funnel::Source<'a> for DetectSource {
  fn register(&'a self, select: &mut Select<'a>) -> usize {
    select.recv(&self.receiver)
  }

  fn receive(&self, op: SelectedOperation) -> Event {
    let (input_event, source_id) = op
      .recv(&self.receiver)
      .expect("unable to select data from DetectSource receiver");
    match input_event {
      InputEvent::Keyboard(keyboard_event) => Event {
        source_id,
        etype: EventType::Keyboard(KeyboardEvent {
          key: convert_to_engine_key(keyboard_event.key),
          value: keyboard_event.value,
          status: convert_to_engine_status(keyboard_event.status),
          variant: keyboard_event
            .variant
            .map(|variant| convert_to_engine_variant(variant)),
        }),
      },
      InputEvent::Mouse(mouse_event) => Event {
        source_id,
        etype: EventType::Mouse(MouseEvent {
          status: convert_to_engine_status(mouse_event.status),
          button: convert_to_engine_mouse_button(mouse_event.button),
        }),
      },
      InputEvent::HotKey(hotkey_event) => Event {
        source_id,
        etype: EventType::HotKey(HotKeyEvent {
          hotkey_id: hotkey_event.hotkey_id,
        }),
      },
    }
  }
}

pub fn convert_to_engine_key(key: espanso_detect::event::Key) -> Key {
  match key {
    espanso_detect::event::Key::Alt => Key::Alt,
    espanso_detect::event::Key::CapsLock => Key::CapsLock,
    espanso_detect::event::Key::Control => Key::Control,
    espanso_detect::event::Key::Meta => Key::Meta,
    espanso_detect::event::Key::NumLock => Key::NumLock,
    espanso_detect::event::Key::Shift => Key::Shift,
    espanso_detect::event::Key::Enter => Key::Enter,
    espanso_detect::event::Key::Tab => Key::Tab,
    espanso_detect::event::Key::Space => Key::Space,
    espanso_detect::event::Key::ArrowDown => Key::ArrowDown,
    espanso_detect::event::Key::ArrowLeft => Key::ArrowLeft,
    espanso_detect::event::Key::ArrowRight => Key::ArrowRight,
    espanso_detect::event::Key::ArrowUp => Key::ArrowUp,
    espanso_detect::event::Key::End => Key::End,
    espanso_detect::event::Key::Home => Key::Home,
    espanso_detect::event::Key::PageDown => Key::PageDown,
    espanso_detect::event::Key::PageUp => Key::PageUp,
    espanso_detect::event::Key::Escape => Key::Escape,
    espanso_detect::event::Key::Backspace => Key::Backspace,
    espanso_detect::event::Key::F1 => Key::F1,
    espanso_detect::event::Key::F2 => Key::F2,
    espanso_detect::event::Key::F3 => Key::F3,
    espanso_detect::event::Key::F4 => Key::F4,
    espanso_detect::event::Key::F5 => Key::F5,
    espanso_detect::event::Key::F6 => Key::F6,
    espanso_detect::event::Key::F7 => Key::F7,
    espanso_detect::event::Key::F8 => Key::F8,
    espanso_detect::event::Key::F9 => Key::F9,
    espanso_detect::event::Key::F10 => Key::F10,
    espanso_detect::event::Key::F11 => Key::F11,
    espanso_detect::event::Key::F12 => Key::F12,
    espanso_detect::event::Key::F13 => Key::F13,
    espanso_detect::event::Key::F14 => Key::F14,
    espanso_detect::event::Key::F15 => Key::F15,
    espanso_detect::event::Key::F16 => Key::F16,
    espanso_detect::event::Key::F17 => Key::F17,
    espanso_detect::event::Key::F18 => Key::F18,
    espanso_detect::event::Key::F19 => Key::F19,
    espanso_detect::event::Key::F20 => Key::F20,
    espanso_detect::event::Key::Other(code) => Key::Other(code),
  }
}

pub fn convert_to_engine_variant(variant: espanso_detect::event::Variant) -> Variant {
  match variant {
    espanso_detect::event::Variant::Left => Variant::Left,
    espanso_detect::event::Variant::Right => Variant::Right,
  }
}

pub fn convert_to_engine_status(status: espanso_detect::event::Status) -> Status {
  match status {
    espanso_detect::event::Status::Pressed => Status::Pressed,
    espanso_detect::event::Status::Released => Status::Released,
  }
}

pub fn convert_to_engine_mouse_button(button: espanso_detect::event::MouseButton) -> MouseButton {
  match button {
    espanso_detect::event::MouseButton::Left => MouseButton::Left,
    espanso_detect::event::MouseButton::Right => MouseButton::Right,
    espanso_detect::event::MouseButton::Middle => MouseButton::Middle,
    espanso_detect::event::MouseButton::Button1 => MouseButton::Button1,
    espanso_detect::event::MouseButton::Button2 => MouseButton::Button2,
    espanso_detect::event::MouseButton::Button3 => MouseButton::Button3,
    espanso_detect::event::MouseButton::Button4 => MouseButton::Button4,
    espanso_detect::event::MouseButton::Button5 => MouseButton::Button5,
  }
}
