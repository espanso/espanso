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

use std::ffi::{c_void, CStr};

use lazycell::LazyCell;
use log::{error, trace, warn};

use anyhow::Result;
use thiserror::Error;

use crate::{Source, SourceCallback, event::Status::*};
use crate::event::Variant::*;
use crate::event::{InputEvent, Key, KeyboardEvent, Variant};
use crate::event::{Key::*, MouseButton, MouseEvent};

const INPUT_EVENT_TYPE_KEYBOARD: i32 = 1;
const INPUT_EVENT_TYPE_MOUSE: i32 = 2;

const INPUT_STATUS_PRESSED: i32 = 1;
const INPUT_STATUS_RELEASED: i32 = 2;

const INPUT_MOUSE_LEFT_BUTTON: i32 = 1;
const INPUT_MOUSE_RIGHT_BUTTON: i32 = 3;
const INPUT_MOUSE_MIDDLE_BUTTON: i32 = 2;
const INPUT_MOUSE_BUTTON_1: i32 = 9;
const INPUT_MOUSE_BUTTON_2: i32 = 8;

// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
pub struct RawInputEvent {
  pub event_type: i32,

  pub buffer: [u8; 24],
  pub buffer_len: i32,

  pub key_sym: i32,
  pub key_code: i32,
  pub status: i32,
}

#[allow(improper_ctypes)]
#[link(name = "espansodetect", kind = "static")]
extern "C" {
  pub fn detect_check_x11() -> i32;

  pub fn detect_initialize(_self: *const X11Source, error_code: *mut i32) -> *mut c_void;

  pub fn detect_eventloop(
    window: *const c_void,
    event_callback: extern "C" fn(_self: *mut X11Source, event: RawInputEvent),
  ) -> i32;

  pub fn detect_destroy(window: *const c_void) -> i32;
}

pub struct X11Source {
  handle: *mut c_void,
  callback: LazyCell<SourceCallback>,
}

#[allow(clippy::new_without_default)]
impl X11Source {
  pub fn new() -> X11Source {
    Self {
      handle: std::ptr::null_mut(),
      callback: LazyCell::new(),
    }
  }

  pub fn is_compatible() -> bool {
    unsafe { detect_check_x11() != 0 }
  }

  
}

impl Source for X11Source {
  fn initialize(&mut self) -> Result<()> {
    let mut error_code = 0;
    let handle = unsafe { detect_initialize(self as *const X11Source, &mut error_code) };

    if handle.is_null() {
      let error = match error_code {
        -1 => X11SourceError::DisplayFailure(),
        -2 => X11SourceError::XRecordMissing(),
        -3 => X11SourceError::XKeyboardMissing(),
        -4 => X11SourceError::FailedRegistration("cannot initialize record range".to_owned()),
        -5 => X11SourceError::FailedRegistration("cannot initialize XRecord context".to_owned()),
        -6 => X11SourceError::FailedRegistration("cannot enable XRecord context".to_owned()),
        _ => X11SourceError::Unknown(),
      };
      return Err(error.into());
    }

    self.handle = handle;

    Ok(())
  }

  fn eventloop(&self, event_callback: SourceCallback) {
    if self.handle.is_null() {
      panic!("Attempt to start X11Source eventloop without initialization");
    }

    if self.callback.fill(event_callback).is_err() {
      panic!("Unable to set X11Source event callback");
    }

    extern "C" fn callback(_self: *mut X11Source, event: RawInputEvent) {
      let event: Option<InputEvent> = event.into();
      if let Some(callback) = unsafe { (*_self).callback.borrow() } {
        if let Some(event) = event {
          callback(event)
        } else {
          trace!("Unable to convert raw event to input event");
        }
      }
    }

    let error_code = unsafe { detect_eventloop(self.handle, callback) };

    if error_code <= 0 {
      panic!("X11Source eventloop returned a negative error code");
    }
  }
}

impl Drop for X11Source {
  fn drop(&mut self) {
    if self.handle.is_null() {
      error!("X11Source destruction cannot be performed, handle is null");
      return;
    }

    let result = unsafe { detect_destroy(self.handle) };

    if result != 0 {
      error!("X11Source destruction returned non-zero code");
    }
  }
}

#[derive(Error, Debug)]
pub enum X11SourceError {
  #[error("cannot open displays")]
  DisplayFailure(),

  #[error("X Record Extension is not installed")]
  XRecordMissing(),

  #[error("X Keyboard Extension is not installed")]
  XKeyboardMissing(),

  #[error("failed registration: ${0}")]
  FailedRegistration(String),

  #[error("unknown error")]
  Unknown(),
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
        let (key, variant) = key_sym_to_key(raw.key_sym);
        let value = if raw.buffer_len > 0 {
          let raw_string_result =
            CStr::from_bytes_with_nul(&raw.buffer[..((raw.buffer_len + 1) as usize)]);
          match raw_string_result {
            Ok(c_string) => {
              let string_result = c_string.to_str();
              match string_result {
                Ok(value) => Some(value.to_string()),
                Err(err) => {
                  warn!("char conversion error: {}", err);
                  None
                }
              }
            }
            Err(err) => {
              warn!("Received malformed char: {}", err);
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
// TODO: might need to add also the variants
fn key_sym_to_key(key_sym: i32) -> (Key, Option<Variant>) {
  match key_sym {
    // Modifiers
    0xFFE9 => (Alt, Some(Left)),
    0xFFEA => (Alt, Some(Right)),
    0xFFE5 => (CapsLock, None),
    0xFFE3 => (Control, Some(Left)),
    0xFFE4 => (Control, Some(Right)),
    0xFFE7 | 0xFFEB => (Meta, Some(Left)),
    0xFFE8 | 0xFFEC => (Meta, Some(Right)),
    0xFF7F => (NumLock, None),
    0xFFE1 => (Shift, Some(Left)),
    0xFFE2 => (Shift, Some(Right)),

    // Whitespace
    0xFF0D => (Enter, None),
    0xFF09 => (Tab, None),
    0x20 => (Space, None),

    // Navigation
    0xFF54 => (ArrowDown, None),
    0xFF51 => (ArrowLeft, None),
    0xFF53 => (ArrowRight, None),
    0xFF52 => (ArrowUp, None),
    0xFF57 => (End, None),
    0xFF50 => (Home, None),
    0xFF56 => (PageDown, None),
    0xFF55 => (PageUp, None),

    // UI keys
    0xFF1B => (Escape, None),

    // Editing keys
    0xFF08 => (Backspace, None),

    // Function keys
    0xFFBE => (F1, None),
    0xFFBF => (F2, None),
    0xFFC0 => (F3, None),
    0xFFC1 => (F4, None),
    0xFFC2 => (F5, None),
    0xFFC3 => (F6, None),
    0xFFC4 => (F7, None),
    0xFFC5 => (F8, None),
    0xFFC6 => (F9, None),
    0xFFC7 => (F10, None),
    0xFFC8 => (F11, None),
    0xFFC9 => (F12, None),
    0xFFCA => (F13, None),
    0xFFCB => (F14, None),
    0xFFCC => (F15, None),
    0xFFCD => (F16, None),
    0xFFCE => (F17, None),
    0xFFCF => (F18, None),
    0xFFD0 => (F19, None),
    0xFFD1 => (F20, None),

    // Other keys, includes the raw code provided by the operating system
    _ => (Other(key_sym), None),
  }
}

fn raw_to_mouse_button(raw: i32) -> Option<MouseButton> {
  match raw {
    INPUT_MOUSE_LEFT_BUTTON => Some(MouseButton::Left),
    INPUT_MOUSE_RIGHT_BUTTON => Some(MouseButton::Right),
    INPUT_MOUSE_MIDDLE_BUTTON => Some(MouseButton::Middle),
    INPUT_MOUSE_BUTTON_1 => Some(MouseButton::Button1),
    INPUT_MOUSE_BUTTON_2 => Some(MouseButton::Button2),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use super::*;

  fn default_raw_input_event() -> RawInputEvent {
    RawInputEvent {
      event_type: INPUT_EVENT_TYPE_KEYBOARD,
      buffer: [0; 24],
      buffer_len: 0,
      key_code: 0,
      key_sym: 0,
      status: INPUT_STATUS_PRESSED,
    }
  }

  #[test]
  fn raw_to_input_event_keyboard_works_correctly() {
    let c_string = CString::new("k".to_string()).unwrap();
    let mut buffer: [u8; 24] = [0; 24];
    buffer[..1].copy_from_slice(c_string.as_bytes());

    let mut raw = default_raw_input_event();
    raw.buffer = buffer;
    raw.buffer_len = 1;
    raw.status = INPUT_STATUS_RELEASED;
    raw.key_sym = 0x4B;

    let result: Option<InputEvent> = raw.into();
    assert_eq!(
      result.unwrap(),
      InputEvent::Keyboard(KeyboardEvent {
        key: Other(0x4B),
        status: Released,
        value: Some("k".to_string()),
        variant: None,
      })
    );
  }

  #[test]
  fn raw_to_input_event_mouse_works_correctly() {
    let mut raw = default_raw_input_event();
    raw.event_type = INPUT_EVENT_TYPE_MOUSE;
    raw.status = INPUT_STATUS_RELEASED;
    raw.key_code = INPUT_MOUSE_RIGHT_BUTTON;

    let result: Option<InputEvent> = raw.into();
    assert_eq!(
      result.unwrap(),
      InputEvent::Mouse(MouseEvent {
        status: Released,
        button: MouseButton::Right,
      })
    );
  }

  #[test]
  fn raw_to_input_invalid_buffer() {
    let buffer: [u8; 24] = [123; 24];

    let mut raw = default_raw_input_event();
    raw.buffer = buffer;
    raw.buffer_len = 5;

    let result: Option<InputEvent> = raw.into();
    assert!(result.unwrap().into_keyboard().unwrap().value.is_none());
  }

  #[test]
  fn raw_to_input_event_returns_none_when_missing_type() {
    let mut raw = default_raw_input_event();
    raw.event_type = 0;
    let result: Option<InputEvent> = raw.into();
    assert!(result.is_none());
  }
}
