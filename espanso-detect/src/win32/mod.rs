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

use log::{trace, warn};
use widestring::U16CStr;

use crate::event::Status::*;
use crate::event::Variant::*;
use crate::event::{InputEvent, Key, KeyboardEvent, Variant};
use crate::event::{Key::*, MouseButton, MouseEvent};

const INPUT_LEFT_VARIANT: i32 = 1;
const INPUT_RIGHT_VARIANT: i32 = 2;

const INPUT_EVENT_TYPE_KEYBOARD: i32 = 1;
const INPUT_EVENT_TYPE_MOUSE: i32 = 2;

const INPUT_STATUS_PRESSED: i32 = 1;
const INPUT_STATUS_RELEASED: i32 = 2;

const INPUT_MOUSE_LEFT_BUTTON: i32 = 1;
const INPUT_MOUSE_RIGHT_BUTTON: i32 = 2;
const INPUT_MOUSE_MIDDLE_BUTTON: i32 = 3;
const INPUT_MOUSE_BUTTON_1: i32 = 4;
const INPUT_MOUSE_BUTTON_2: i32 = 5;
const INPUT_MOUSE_BUTTON_3: i32 = 6;
const INPUT_MOUSE_BUTTON_4: i32 = 7;
const INPUT_MOUSE_BUTTON_5: i32 = 8;

// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
pub struct RawInputEvent {
  pub event_type: i32,

  pub buffer: [u16; 24],
  pub buffer_len: i32,

  pub key_code: i32,
  pub variant: i32,
  pub status: i32,
}

#[allow(improper_ctypes)]
#[link(name = "native", kind = "static")]
extern "C" {
  pub fn raw_eventloop(
    _self: *const Win32Source,
    event_callback: extern "C" fn(_self: *mut Win32Source, event: RawInputEvent),
  );
}

pub type Win32SourceCallback = Box<dyn Fn(InputEvent)>;
pub struct Win32Source {
  callback: Win32SourceCallback,
}

impl Win32Source {
  pub fn new(callback: Win32SourceCallback) -> Win32Source {
    Self { callback }
  }
  pub fn eventloop(&self) {
    unsafe {
      extern "C" fn callback(_self: *mut Win32Source, event: RawInputEvent) {
        let event: Option<InputEvent> = event.into();
        if let Some(event) = event {
          unsafe { (*(*_self).callback)(event) }
        } else {
          trace!("Unable to convert raw event to input event");
        }
      }

      raw_eventloop(self as *const Win32Source, callback);
    }
  }
}

impl From<RawInputEvent> for Option<InputEvent> {
  fn from(raw: RawInputEvent) -> Option<InputEvent> {
    let status = match raw.status {
      INPUT_STATUS_RELEASED => Released,
      INPUT_STATUS_PRESSED => Pressed,
      _ => Pressed,
    };

    match raw.event_type {
      // Keyboard events
      INPUT_EVENT_TYPE_KEYBOARD => {
        let (key, variant_hint) = key_code_to_key(raw.key_code);

        // If the raw event does not include an explicit variant, use the hint provided by the key code
        let variant = match raw.variant {
          INPUT_LEFT_VARIANT => Some(Left),
          INPUT_RIGHT_VARIANT => Some(Right),
          _ => variant_hint,
        };

        let value = if raw.buffer_len > 0 {
          let raw_string_result = U16CStr::from_slice_with_nul(&raw.buffer);
          match raw_string_result {
            Ok(c_string) => {
              let string_result = c_string.to_string();
              match string_result {
                Ok(value) => Some(value),
                Err(err) => {
                  warn!("Widechar conversion error: {}", err);
                  None
                }
              }
            }
            Err(err) => {
              warn!("Received malformed widechar: {}", err);
              None
            }
          }
        } else {
          None
        };

        return Some(InputEvent::Keyboard(KeyboardEvent {
          key,
          value,
          status,
          variant,
        }));
      }
      // Mouse events
      INPUT_EVENT_TYPE_MOUSE => {
        let button = raw_to_mouse_button(raw.key_code);

        if let Some(button) = button {
          return Some(InputEvent::Mouse(MouseEvent { button, status }));
        }
      }
      _ => {}
    }

    None
  }
}

// Mappings from: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values
fn key_code_to_key(key_code: i32) -> (Key, Option<Variant>) {
  match key_code {
    // Modifiers
    0x12 => (Alt, None),
    0xA4 => (Alt, Some(Left)),
    0xA5 => (Alt, Some(Right)),
    0x14 => (CapsLock, None),
    0x11 => (Control, None),
    0xA2 => (Control, Some(Left)),
    0xA3 => (Control, Some(Right)),
    0x5B => (Meta, Some(Left)),
    0x5C => (Meta, Some(Right)),
    0x90 => (NumLock, None),
    0x10 => (Shift, None),
    0xA0 => (Shift, Some(Left)),
    0xA1 => (Shift, Some(Right)),

    // Whitespace
    0x0D => (Enter, None),
    0x09 => (Tab, None),
    0x20 => (Space, None),

    // Navigation
    0x28 => (ArrowDown, None),
    0x25 => (ArrowLeft, None),
    0x27 => (ArrowRight, None),
    0x26 => (ArrowUp, None),
    0x23 => (End, None),
    0x24 => (Home, None),
    0x22 => (PageDown, None),
    0x21 => (PageUp, None),

    // Editing keys
    0x08 => (Backspace, None),

    // Function keys
    0x70 => (F1, None),
    0x71 => (F2, None),
    0x72 => (F3, None),
    0x73 => (F4, None),
    0x74 => (F5, None),
    0x75 => (F6, None),
    0x76 => (F7, None),
    0x77 => (F8, None),
    0x78 => (F9, None),
    0x79 => (F10, None),
    0x7A => (F11, None),
    0x7B => (F12, None),
    0x7C => (F13, None),
    0x7D => (F14, None),
    0x7E => (F15, None),
    0x7F => (F16, None),
    0x80 => (F17, None),
    0x81 => (F18, None),
    0x82 => (F19, None),
    0x83 => (F20, None),

    // Other keys, includes the raw code provided by the operating system
    _ => (Other(key_code), None),
  }
}

fn raw_to_mouse_button(raw: i32) -> Option<MouseButton> {
  match raw {
    INPUT_MOUSE_LEFT_BUTTON => Some(MouseButton::Left),
    INPUT_MOUSE_RIGHT_BUTTON => Some(MouseButton::Right),
    INPUT_MOUSE_MIDDLE_BUTTON => Some(MouseButton::Middle),
    INPUT_MOUSE_BUTTON_1 => Some(MouseButton::Button1),
    INPUT_MOUSE_BUTTON_2 => Some(MouseButton::Button2),
    INPUT_MOUSE_BUTTON_3 => Some(MouseButton::Button3),
    INPUT_MOUSE_BUTTON_4 => Some(MouseButton::Button4),
    INPUT_MOUSE_BUTTON_5 => Some(MouseButton::Button5),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn default_raw_input_event() -> RawInputEvent {
    RawInputEvent {
      event_type: INPUT_EVENT_TYPE_KEYBOARD,
      buffer: [0; 24],
      buffer_len: 0,
      key_code: 0,
      variant: INPUT_LEFT_VARIANT,
      status: INPUT_STATUS_PRESSED,
    }
  }

  #[test]
  fn raw_to_input_event_keyboard_works_correctly() {
    let wide_string = widestring::WideString::from("k".to_string());
    let mut buffer: [u16; 24] = [0; 24];
    buffer[..1].copy_from_slice(wide_string.as_slice());

    let mut raw = default_raw_input_event();
    raw.buffer = buffer;
    raw.buffer_len = 1;
    raw.status = INPUT_STATUS_RELEASED;
    raw.variant = 0;
    raw.key_code = 0x4B;

    let result: Option<InputEvent> = raw.into();
    assert_eq!(result.unwrap(), InputEvent::Keyboard(KeyboardEvent {
      key: Other(0x4B),
      status: Released,
      value: Some("k".to_string()),
      variant: None,
    }));
  }

  #[test]
  fn raw_to_input_event_mouse_works_correctly() {

    let mut raw = default_raw_input_event();
    raw.event_type = INPUT_EVENT_TYPE_MOUSE;
    raw.status = INPUT_STATUS_RELEASED;
    raw.variant = 0;
    raw.key_code = INPUT_MOUSE_RIGHT_BUTTON;

    let result: Option<InputEvent> = raw.into();
    assert_eq!(result.unwrap(), InputEvent::Mouse(MouseEvent {
      status: Released,
      button: MouseButton::Right,
    }));
  }

  #[test]
  fn raw_to_input_invalid_buffer() {
    let buffer: [u16; 24] = [123; 24];

    let mut raw = default_raw_input_event();
    raw.buffer = buffer;
    raw.buffer_len = 5;

    let result: Option<InputEvent> = raw.into();
    assert!(result.unwrap().into_keyboard().unwrap().value.is_none());
  }

  #[test]
  fn raw_to_input_event_returns_none_when_missing_type() {
    let result: Option<InputEvent> = RawInputEvent {
      event_type: 0,  // Missing type
      buffer: [0; 24],
      buffer_len: 0,
      key_code: 123,
      variant: INPUT_LEFT_VARIANT,
      status: INPUT_STATUS_PRESSED,
    }.into();
    assert!(result.is_none());
  }
}
