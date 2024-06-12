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

use std::os::raw::c_long;
use std::{convert::TryInto, ffi::c_void};

use lazycell::LazyCell;
use log::{debug, error, trace, warn};
use widestring::U16CStr;

use anyhow::Result;
use thiserror::Error;

use crate::event::{
  HotKeyEvent, InputEvent,
  Key::{
    self, Alt, ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Backspace, CapsLock, Control, End, Enter,
    Escape, Home, Meta, NumLock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6,
    Numpad7, Numpad8, Numpad9, Other, PageDown, PageUp, Shift, Space, Tab, F1, F10, F11, F12, F13,
    F14, F15, F16, F17, F18, F19, F2, F20, F3, F4, F5, F6, F7, F8, F9,
  },
  KeyboardEvent, MouseButton, MouseEvent,
  Status::{Pressed, Released},
  Variant::{self, Left, Right},
};
use crate::hotkey::HotKey;
use crate::{Source, SourceCallback};

const INPUT_LEFT_VARIANT: i32 = 1;
const INPUT_RIGHT_VARIANT: i32 = 2;

const INPUT_EVENT_TYPE_KEYBOARD: i32 = 1;
const INPUT_EVENT_TYPE_MOUSE: i32 = 2;
const INPUT_EVENT_TYPE_HOTKEY: i32 = 3;

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

  // Only relevant for keyboard events, this is set to 1
  // if a keyboard event has an explicit source, 0 otherwise.
  // This is needed to filter out software generated events,
  // including those from espanso.
  pub has_known_source: i32,
}

#[repr(C)]
pub struct RawHotKey {
  pub id: i32,
  pub code: u32,
  pub flags: u32,
}

#[repr(C)]
pub struct InitOptions {
  pub keyboard_layout_cache_interval: c_long,
}

#[allow(improper_ctypes)]
#[link(name = "espansodetect", kind = "static")]
extern "C" {
  pub fn detect_initialize(
    _self: *const Win32Source,
    options: *const InitOptions,
    error_code: *mut i32,
  ) -> *mut c_void;
  pub fn detect_register_hotkey(window: *const c_void, hotkey: RawHotKey) -> i32;

  pub fn detect_eventloop(
    window: *const c_void,
    event_callback: extern "C" fn(_self: *mut Win32Source, event: RawInputEvent),
  ) -> i32;

  pub fn detect_destroy(window: *const c_void) -> i32;
}

pub struct Win32Source {
  handle: *mut c_void,
  callback: LazyCell<SourceCallback>,
  hotkeys: Vec<HotKey>,

  exclude_orphan_events: bool,
  keyboard_layout_cache_interval: i64,
}

#[allow(clippy::new_without_default)]
impl Win32Source {
  pub fn new(
    hotkeys: &[HotKey],
    exclude_orphan_events: bool,
    keyboard_layout_cache_interval: i64,
  ) -> Win32Source {
    Self {
      handle: std::ptr::null_mut(),
      callback: LazyCell::new(),
      hotkeys: hotkeys.to_vec(),
      exclude_orphan_events,
      keyboard_layout_cache_interval,
    }
  }
}

impl Source for Win32Source {
  fn initialize(&mut self) -> Result<()> {
    let options = InitOptions {
      keyboard_layout_cache_interval: self.keyboard_layout_cache_interval.try_into().unwrap(),
    };

    let mut error_code = 0;
    let handle = unsafe {
      detect_initialize(
        std::ptr::from_ref::<Win32Source>(self),
        &options,
        &mut error_code,
      )
    };

    if handle.is_null() {
      let error = match error_code {
        -1 => Win32SourceError::WindowFailed(),
        -2 => Win32SourceError::RawInputFailed(),
        _ => Win32SourceError::Unknown(),
      };
      return Err(error.into());
    }

    // Register the hotkeys
    self.hotkeys.iter().for_each(|hk| {
      let raw = convert_hotkey_to_raw(hk);
      if let Some(raw_hk) = raw {
        if unsafe { detect_register_hotkey(handle, raw_hk) } == 0 {
          error!("unable to register hotkey: {}", hk);
        } else {
          debug!("registered hotkey: {}", hk);
        }
      } else {
        error!("unable to generate raw hotkey mapping: {}", hk);
      }
    });

    self.handle = handle;

    Ok(())
  }

  fn eventloop(&self, event_callback: SourceCallback) -> Result<()> {
    assert!(
      !self.handle.is_null(),
      "Attempt to start Win32Source eventloop without initialization"
    );

    if self.callback.fill(event_callback).is_err() {
      error!("Unable to set Win32Source event callback");
      return Err(Win32SourceError::Unknown().into());
    }

    extern "C" fn callback(_self: *mut Win32Source, event: RawInputEvent) {
      // Filter out keyboard events without an explicit HID device source.
      // This is needed to filter out the software-generated events, including
      // those from espanso.
      if event.event_type == INPUT_EVENT_TYPE_KEYBOARD
        && event.has_known_source == 0
        && unsafe { (*_self).exclude_orphan_events }
      {
        trace!("skipping keyboard event with unknown HID source (probably software generated).");
        return;
      }

      let event: Option<InputEvent> = event.into();
      if let Some(callback) = unsafe { (*_self).callback.borrow() } {
        if let Some(event) = event {
          callback(event);
        } else {
          trace!("Unable to convert raw event to input event");
        }
      }
    }

    let error_code = unsafe { detect_eventloop(self.handle, callback) };

    if error_code <= 0 {
      error!("Win32Source eventloop returned a negative error code");
      return Err(Win32SourceError::Unknown().into());
    }

    Ok(())
  }
}

impl Drop for Win32Source {
  fn drop(&mut self) {
    if self.handle.is_null() {
      error!("Win32Source destruction cannot be performed, handle is null");
      return;
    }

    let result = unsafe { detect_destroy(self.handle) };

    if result != 0 {
      error!("Win32Source destruction returned non-zero code");
    }
  }
}

fn convert_hotkey_to_raw(hk: &HotKey) -> Option<RawHotKey> {
  let key_code = hk.key.to_code()?;
  let mut flags = 0x4000; // NOREPEAT flags
  if hk.has_ctrl() {
    flags |= 0x0002;
  }
  if hk.has_alt() {
    flags |= 0x0001;
  }
  if hk.has_meta() {
    flags |= 0x0008;
  }
  if hk.has_shift() {
    flags |= 0x0004;
  }

  Some(RawHotKey {
    id: hk.id,
    code: key_code,
    flags,
  })
}

#[derive(Error, Debug)]
pub enum Win32SourceError {
  #[error("window registration failed")]
  WindowFailed(),

  #[error("raw input API failed")]
  RawInputFailed(),

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
          code: raw
            .key_code
            .try_into()
            .expect("unable to convert keycode to u32"),
        }));
      }
      // Mouse events
      INPUT_EVENT_TYPE_MOUSE => {
        let button = raw_to_mouse_button(raw.key_code);

        if let Some(button) = button {
          return Some(InputEvent::Mouse(MouseEvent { button, status }));
        }
      }
      // Hotkey events
      INPUT_EVENT_TYPE_HOTKEY => {
        return Some(InputEvent::HotKey(HotKeyEvent {
          hotkey_id: raw.key_code,
        }))
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

    // UI
    0x1B => (Escape, None),

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

    // Numpad
    0x60 => (Numpad0, None),
    0x61 => (Numpad1, None),
    0x62 => (Numpad2, None),
    0x63 => (Numpad3, None),
    0x64 => (Numpad4, None),
    0x65 => (Numpad5, None),
    0x66 => (Numpad6, None),
    0x67 => (Numpad7, None),
    0x68 => (Numpad8, None),
    0x69 => (Numpad9, None),

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
      has_known_source: 1,
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
    assert_eq!(
      result.unwrap(),
      InputEvent::Keyboard(KeyboardEvent {
        key: Other(0x4B),
        status: Released,
        value: Some("k".to_string()),
        variant: None,
        code: 0x4B,
      })
    );
  }

  #[test]
  fn raw_to_input_event_mouse_works_correctly() {
    let mut raw = default_raw_input_event();
    raw.event_type = INPUT_EVENT_TYPE_MOUSE;
    raw.status = INPUT_STATUS_RELEASED;
    raw.variant = 0;
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
      event_type: 0, // Missing type
      buffer: [0; 24],
      buffer_len: 0,
      key_code: 123,
      variant: INPUT_LEFT_VARIANT,
      status: INPUT_STATUS_PRESSED,
      has_known_source: 1,
    }
    .into();
    assert!(result.is_none());
  }
}
