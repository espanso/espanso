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

use anyhow::Result;
use lazy_static::lazy_static;
use lazycell::LazyCell;
use log::{error, trace, warn};
use std::{
  convert::TryInto,
  ffi::CStr,
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
  },
};
use thiserror::Error;

use crate::event::{HotKeyEvent, InputEvent, Key, KeyboardEvent, Status, Variant};
use crate::event::{
  Key::{
    Alt, ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Backspace, CapsLock, Control, End, Enter,
    Escape, Home, Meta, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7,
    Numpad8, Numpad9, Other, PageDown, PageUp, Shift, Space, Tab, F1, F10, F11, F12, F13, F14, F15,
    F16, F17, F18, F19, F2, F20, F3, F4, F5, F6, F7, F8, F9,
  },
  MouseButton, MouseEvent,
};
use crate::{
  event::Status::{Pressed, Released},
  Source, SourceCallback,
};
use crate::{
  event::Variant::{Left, Right},
  hotkey::HotKey,
};

const INPUT_EVENT_TYPE_KEYBOARD: i32 = 1;
const INPUT_EVENT_TYPE_MOUSE: i32 = 2;
const INPUT_EVENT_TYPE_HOTKEY: i32 = 3;

const INPUT_STATUS_PRESSED: i32 = 1;
const INPUT_STATUS_RELEASED: i32 = 2;

const INPUT_MOUSE_LEFT_BUTTON: i32 = 1;
const INPUT_MOUSE_RIGHT_BUTTON: i32 = 2;
const INPUT_MOUSE_MIDDLE_BUTTON: i32 = 3;

// Take a look at the native.h header file for an explanation of the fields
#[repr(C)]
#[derive(Debug)]
pub struct RawInputEvent {
  pub event_type: i32,

  pub buffer: [u8; 24],
  pub buffer_len: i32,

  pub key_code: i32,
  pub status: i32,

  pub is_caps_lock_pressed: i32,
  pub is_shift_pressed: i32,
  pub is_control_pressed: i32,
  pub is_option_pressed: i32,
  pub is_command_pressed: i32,
}

#[repr(C)]
pub struct RawHotKey {
  pub id: i32,
  pub code: u16,
  pub flags: u32,
}

#[repr(C)]
pub struct RawInitializationOptions {
  pub hotkeys: *const RawHotKey,
  pub hotkeys_count: i32,
}

#[allow(improper_ctypes)]
#[link(name = "espansodetect", kind = "static")]
extern "C" {
  pub fn detect_initialize(
    callback: extern "C" fn(event: RawInputEvent),
    options: RawInitializationOptions,
  );
}

#[derive(Debug, Default)]
struct ModifierState {
  is_ctrl_down: bool,
  is_shift_down: bool,
  is_command_down: bool,
  is_option_down: bool,
}

lazy_static! {
  static ref CURRENT_SENDER: Arc<Mutex<Option<Sender<InputEvent>>>> = Arc::new(Mutex::new(None));
  static ref MODIFIER_STATE: Arc<Mutex<ModifierState>> =
    Arc::new(Mutex::new(ModifierState::default()));
}

extern "C" fn native_callback(raw_event: RawInputEvent) {
  let lock = CURRENT_SENDER
    .lock()
    .expect("unable to acquire CocoaSource sender lock");

  // Most of the times, when pressing a modifier key (such as Alt, Ctrl, Shift, Cmd),
  // we get both a Pressed and Released event. This is important to keep Espanso's
  // internal representation of modifiers in sync.
  // Unfortunately, there are times when the corresponding "release" event is not sent,
  // and this causes Espanso to mistakenly think that the modifier is still pressed.
  // This can happen for various reasons, such as when using external bluetooth keyboards
  // or certain keyboard shortcuts.
  // Luckily, most key events include the "modifiers flag" information, that tells us which
  // modifier keys were currently pressed at that time.
  // We use this modifier flag information to detect "inconsistent" states to send the corresponding
  // modifier release events, keeping espanso's state in sync.
  // For more info, see:
  // https://github.com/espanso/espanso/issues/825
  // https://github.com/espanso/espanso/issues/858
  let mut compensating_events = Vec::new();
  if raw_event.event_type == INPUT_EVENT_TYPE_KEYBOARD {
    let (key_code, _) = key_code_to_key(raw_event.key_code);
    let mut current_mod_state = MODIFIER_STATE
      .lock()
      .expect("unable to acquire modifier state in cocoa detector");

    if let Key::Alt = &key_code {
      current_mod_state.is_option_down = raw_event.status == INPUT_STATUS_PRESSED;
    } else if let Key::Meta = &key_code {
      current_mod_state.is_command_down = raw_event.status == INPUT_STATUS_PRESSED;
    } else if let Key::Shift = &key_code {
      current_mod_state.is_shift_down = raw_event.status == INPUT_STATUS_PRESSED;
    } else if let Key::Control = &key_code {
      current_mod_state.is_ctrl_down = raw_event.status == INPUT_STATUS_PRESSED;
    } else {
      if current_mod_state.is_command_down && raw_event.is_command_pressed == 0 {
        compensating_events.push((Key::Meta, 0x37));
        current_mod_state.is_command_down = false;
      }
      if current_mod_state.is_ctrl_down && raw_event.is_control_pressed == 0 {
        compensating_events.push((Key::Control, 0x3B));
        current_mod_state.is_ctrl_down = false;
      }
      if current_mod_state.is_shift_down && raw_event.is_shift_pressed == 0 {
        compensating_events.push((Key::Shift, 0x38));
        current_mod_state.is_shift_down = false;
      }
      if current_mod_state.is_option_down && raw_event.is_option_pressed == 0 {
        compensating_events.push((Key::Alt, 0x3A));
        current_mod_state.is_option_down = false;
      }
    }
  }

  if !compensating_events.is_empty() {
    warn!(
      "detected inconsistent modifier state for keys {:?}, sending compensating events...",
      compensating_events
    );
  }

  if let Some(sender) = lock.as_ref() {
    for (key, code) in compensating_events {
      if let Err(error) = sender.send(InputEvent::Keyboard(KeyboardEvent {
        key,
        value: None,
        status: Status::Released,
        variant: None,
        code,
      })) {
        error!(
          "Unable to send compensating event to Cocoa Sender: {}",
          error
        );
      }
    }

    let event: Option<InputEvent> = raw_event.into();
    if let Some(event) = event {
      if let Err(error) = sender.send(event) {
        error!("Unable to send event to Cocoa Sender: {}", error);
      }
    } else {
      trace!("Unable to convert raw event to input event");
    }
  } else {
    warn!("Lost raw event, as Cocoa Sender is not available");
  }
}

pub struct CocoaSource {
  receiver: LazyCell<Receiver<InputEvent>>,
  hotkeys: Vec<HotKey>,
}

#[allow(clippy::new_without_default)]
impl CocoaSource {
  pub fn new(hotkeys: &[HotKey]) -> CocoaSource {
    Self {
      receiver: LazyCell::new(),
      hotkeys: hotkeys.to_vec(),
    }
  }
}

impl Source for CocoaSource {
  fn initialize(&mut self) -> Result<()> {
    let (sender, receiver) = channel();

    // Set the global sender
    {
      let mut lock = CURRENT_SENDER
        .lock()
        .expect("unable to acquire CocoaSource sender lock during initialization");
      *lock = Some(sender);
    }

    // Generate the options
    let hotkeys: Vec<RawHotKey> = self
      .hotkeys
      .iter()
      .filter_map(|hk| {
        let raw = convert_hotkey_to_raw(hk);
        if raw.is_none() {
          error!("unable to register hotkey: {:?}", hk);
        }
        raw
      })
      .collect();
    let options = RawInitializationOptions {
      hotkeys: hotkeys.as_ptr(),
      hotkeys_count: hotkeys.len() as i32,
    };

    unsafe { detect_initialize(native_callback, options) };

    if self.receiver.fill(receiver).is_err() {
      error!("Unable to set CocoaSource receiver");
      return Err(CocoaSourceError::Unknown().into());
    }

    Ok(())
  }

  fn eventloop(&self, event_callback: SourceCallback) -> Result<()> {
    if let Some(receiver) = self.receiver.borrow() {
      loop {
        let event = receiver.recv();
        match event {
          Ok(event) => {
            event_callback(event);
          }
          Err(error) => {
            error!("CocoaSource receiver reported error: {}", error);
            break;
          }
        }
      }
    } else {
      error!("Unable to start event loop if CocoaSource receiver is null");
      return Err(CocoaSourceError::Unknown().into());
    }

    Ok(())
  }
}

impl Drop for CocoaSource {
  fn drop(&mut self) {
    // Reset the global sender
    {
      let mut lock = CURRENT_SENDER
        .lock()
        .expect("unable to acquire CocoaSource sender lock during initialization");
      *lock = None;
    }
  }
}

fn convert_hotkey_to_raw(hk: &HotKey) -> Option<RawHotKey> {
  let key_code = hk.key.to_code()?;
  let code: Result<u16, _> = key_code.try_into();
  if let Ok(code) = code {
    let mut flags = 0;
    if hk.has_ctrl() {
      flags |= 1 << 12;
    }
    if hk.has_alt() {
      flags |= 1 << 11;
    }
    if hk.has_meta() {
      flags |= 1 << 8;
    }
    if hk.has_shift() {
      flags |= 1 << 9;
    }

    Some(RawHotKey {
      id: hk.id,
      code,
      flags,
    })
  } else {
    error!("unable to generate raw hotkey, the key_code is overflowing");
    None
  }
}

#[derive(Error, Debug)]
pub enum CocoaSourceError {
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
        let (key, variant) = key_code_to_key(raw.key_code);

        // When a global keyboard shortcut is relased, the callback returns an event with keycode 0
        // and status 0.
        // We need to handle it for this reason: https://github.com/espanso/espanso/issues/791
        if raw.key_code == 0 && raw.status == 0 {
          return Some(InputEvent::AllModifiersReleased);
        }

        let value = if raw.buffer_len > 0 {
          let raw_string_result =
            CStr::from_bytes_with_nul(&raw.buffer[..((raw.buffer_len + 1) as usize)]);
          match raw_string_result {
            Ok(c_string) => {
              let string_result = c_string.to_str();
              match string_result {
                Ok(value) => Some(value.to_string()),
                Err(err) => {
                  warn!("CocoaSource event utf8 conversion error: {}", err);
                  None
                }
              }
            }
            Err(err) => {
              trace!("Received malformed event buffer: {}", err);
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
      // HOTKEYS
      INPUT_EVENT_TYPE_HOTKEY => {
        let id = raw.key_code;
        return Some(InputEvent::HotKey(HotKeyEvent { hotkey_id: id }));
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
    0x3A => (Alt, Some(Left)),
    0x3D => (Alt, Some(Right)),
    0x39 => (CapsLock, None), // TODO
    0x3B => (Control, Some(Left)),
    0x3E => (Control, Some(Right)),
    0x37 => (Meta, Some(Left)),
    0x36 => (Meta, Some(Right)),
    0x38 => (Shift, Some(Left)),
    0x3C => (Shift, Some(Right)),

    // Whitespace
    0x24 => (Enter, None),
    0x30 => (Tab, None),
    0x31 => (Space, None),

    // Navigation
    0x7D => (ArrowDown, None),
    0x7B => (ArrowLeft, None),
    0x7C => (ArrowRight, None),
    0x7E => (ArrowUp, None),
    0x77 => (End, None),
    0x73 => (Home, None),
    0x79 => (PageDown, None),
    0x74 => (PageUp, None),

    // UI
    0x35 => (Escape, None),

    // Editing keys
    0x33 => (Backspace, None),

    // Function keys
    0x7A => (F1, None),
    0x78 => (F2, None),
    0x63 => (F3, None),
    0x76 => (F4, None),
    0x60 => (F5, None),
    0x61 => (F6, None),
    0x62 => (F7, None),
    0x64 => (F8, None),
    0x65 => (F9, None),
    0x6D => (F10, None),
    0x67 => (F11, None),
    0x6F => (F12, None),
    0x69 => (F13, None),
    0x6B => (F14, None),
    0x71 => (F15, None),
    0x6A => (F16, None),
    0x40 => (F17, None),
    0x4F => (F18, None),
    0x50 => (F19, None),
    0x5A => (F20, None),

    // Numpad
    0x52 => (Numpad0, None),
    0x53 => (Numpad1, None),
    0x54 => (Numpad2, None),
    0x55 => (Numpad3, None),
    0x56 => (Numpad4, None),
    0x57 => (Numpad5, None),
    0x58 => (Numpad6, None),
    0x59 => (Numpad7, None),
    0x5B => (Numpad8, None),
    0x5C => (Numpad9, None),

    // Other keys, includes the raw code provided by the operating system
    _ => (Other(key_code), None),
  }
}

fn raw_to_mouse_button(raw: i32) -> Option<MouseButton> {
  match raw {
    INPUT_MOUSE_LEFT_BUTTON => Some(MouseButton::Left),
    INPUT_MOUSE_RIGHT_BUTTON => Some(MouseButton::Right),
    INPUT_MOUSE_MIDDLE_BUTTON => Some(MouseButton::Middle),
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
      status: INPUT_STATUS_PRESSED,
      is_caps_lock_pressed: 0,
      is_shift_pressed: 0,
      is_control_pressed: 0,
      is_option_pressed: 0,
      is_command_pressed: 0,
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
    raw.key_code = 40;

    let result: Option<InputEvent> = raw.into();
    assert_eq!(
      result.unwrap(),
      InputEvent::Keyboard(KeyboardEvent {
        key: Other(40),
        status: Released,
        value: Some("k".to_string()),
        variant: None,
        code: 40,
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
}
