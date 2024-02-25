// This code is heavily inspired by the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

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

mod context;
mod device;
mod ffi;
mod hotkey;
mod keymap;
mod state;
mod sync;

use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::{Context as AnyhowContext, Result};
use context::Context;
use device::{get_devices, Device};
use keymap::Keymap;
use lazycell::LazyCell;
use libc::{
  __errno_location, close, epoll_ctl, epoll_event, epoll_wait, EINTR, EPOLLIN, EPOLL_CTL_ADD,
  EPOLL_CTL_DEL,
};
use log::{debug, error, info, trace, warn};
use thiserror::Error;

use crate::event::{InputEvent, Key, KeyboardEvent, Variant};
use crate::event::{
  Key::{
    Alt, ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Backspace, CapsLock, Control, End, Enter,
    Escape, Home, Meta, NumLock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6,
    Numpad7, Numpad8, Numpad9, Other, PageDown, PageUp, Shift, Space, Tab, F1, F10, F11, F12, F13,
    F14, F15, F16, F17, F18, F19, F2, F20, F3, F4, F5, F6, F7, F8, F9,
  },
  MouseButton, MouseEvent,
};
use crate::{
  event::HotKeyEvent,
  event::Variant::{Left, Right},
  hotkey::HotKey,
};
use crate::{
  event::Status::{Pressed, Released},
  KeyboardConfig, Source, SourceCallback, SourceCreationOptions,
};

use self::{
  device::{DeviceError, RawInputEvent, KEY_STATE_PRESS, KEY_STATE_RELEASE},
  hotkey::HotKeyFilter,
  state::State,
};

const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const BTN_SIDE: u16 = 0x113;
const BTN_EXTRA: u16 = 0x114;

// Offset between evdev keycodes (where KEY_ESCAPE is 1), and the evdev XKB
// keycode set (where ESC is 9).
const EVDEV_OFFSET: u32 = 8;

// List of modifier keycodes, as defined in the "input-event-codes.h" header
// TODO: create an option to override them if needed
const KEY_CTRL: u32 = 29;
const KEY_SHIFT: u32 = 42;
const KEY_ALT: u32 = 56;
const KEY_META: u32 = 125;
const KEY_CAPSLOCK: u32 = 58;
const KEY_NUMLOCK: u32 = 69;

pub struct EVDEVSource {
  devices: Vec<Device>,
  hotkeys: Vec<HotKey>,

  _keyboard_rmlvo: Option<KeyboardConfig>,
  _context: LazyCell<Context>,
  _keymap: LazyCell<Keymap>,
  _hotkey_filter: RefCell<HotKeyFilter>,
  _modifiers_map: HashMap<String, u32>,
}

#[allow(clippy::new_without_default)]
impl EVDEVSource {
  pub fn new(options: SourceCreationOptions) -> EVDEVSource {
    let mut modifiers_map = HashMap::new();
    modifiers_map.insert("ctrl".to_string(), KEY_CTRL + EVDEV_OFFSET);
    modifiers_map.insert("shift".to_string(), KEY_SHIFT + EVDEV_OFFSET);
    modifiers_map.insert("alt".to_string(), KEY_ALT + EVDEV_OFFSET);
    modifiers_map.insert("meta".to_string(), KEY_META + EVDEV_OFFSET);
    modifiers_map.insert("caps_lock".to_string(), KEY_CAPSLOCK + EVDEV_OFFSET);
    modifiers_map.insert("num_lock".to_string(), KEY_NUMLOCK + EVDEV_OFFSET);

    Self {
      devices: Vec::new(),
      hotkeys: options.hotkeys,
      _context: LazyCell::new(),
      _keymap: LazyCell::new(),
      _keyboard_rmlvo: options.evdev_keyboard_rmlvo,
      _hotkey_filter: RefCell::new(HotKeyFilter::new()),
      _modifiers_map: modifiers_map,
    }
  }
}

impl Source for EVDEVSource {
  fn initialize(&mut self) -> Result<()> {
    let context = Context::new().expect("unable to obtain xkb context");
    let keymap =
      Keymap::new(&context, self._keyboard_rmlvo.clone()).expect("unable to create xkb keymap");

    match get_devices(&keymap) {
      Ok(devices) => self.devices = devices,
      Err(error) => {
        if let Some(device_error) = error.downcast_ref::<DeviceError>() {
          if matches!(device_error, DeviceError::NoDevicesFound()) {
            error!("Unable to open EVDEV devices, this usually has to do with permissions.");
            error!(
              "You can either add the current user to the 'input' group or run espanso as root"
            );
            return Err(EVDEVSourceError::PermissionDenied().into());
          }
        }
        return Err(error);
      }
    }

    let state = State::new(&keymap)?;

    info!("Querying modifier status...");
    if let Some(modifiers_state) =
      sync::get_modifiers_state().context("EVDEV modifier context state synchronization")?
    {
      debug!("Updating device modifier state: {:?}", modifiers_state);

      for device in &mut self.devices {
        device.update_modifier_state(modifiers_state, &self._modifiers_map);
      }
    }

    // Initialize the hotkeys
    self
      ._hotkey_filter
      .borrow_mut()
      .initialize(&state, &self.hotkeys);

    if self._context.fill(context).is_err() {
      return Err(EVDEVSourceError::InitFailure().into());
    }
    if self._keymap.fill(keymap).is_err() {
      return Err(EVDEVSourceError::InitFailure().into());
    }

    Ok(())
  }

  fn eventloop(&self, event_callback: SourceCallback) -> Result<()> {
    if self.devices.is_empty() {
      error!("can't start eventloop without evdev devices");
      return Err(EVDEVSourceError::NoDevices().into());
    }

    let raw_epfd = unsafe { libc::epoll_create1(0) };
    let epfd = scopeguard::guard(raw_epfd, |raw_epfd| unsafe {
      close(raw_epfd);
    });

    if *epfd < 0 {
      error!("could not create epoll instance");
      return Err(EVDEVSourceError::Internal().into());
    }

    // Setup epoll for all input devices
    let errno_ptr = unsafe { __errno_location() };
    for (i, device) in self.devices.iter().enumerate() {
      let mut ev: epoll_event = unsafe { std::mem::zeroed() };
      ev.events = EPOLLIN as u32;
      ev.u64 = i as u64;
      if unsafe { epoll_ctl(*epfd, EPOLL_CTL_ADD, device.get_raw_fd(), &mut ev) } != 0 {
        error!(
          "Could not add {} to epoll, errno {}",
          device.get_path(),
          unsafe { *errno_ptr }
        );
        return Err(EVDEVSourceError::Internal().into());
      }
    }

    let mut hotkey_filter = self._hotkey_filter.borrow_mut();

    // Read events indefinitely
    let mut evs: [epoll_event; 16] = unsafe { std::mem::zeroed() };
    loop {
      let ret = unsafe { epoll_wait(*epfd, evs.as_mut_ptr(), 16, -1) };
      if ret < 0 {
        if unsafe { *errno_ptr } == EINTR {
          continue;
        }
        error!("Could not poll for events, {}", unsafe { *errno_ptr });
        return Err(EVDEVSourceError::Internal().into());
      }

      #[allow(clippy::needless_range_loop)]
      for i in 0usize..(ret as usize) {
        let ev = evs[i];
        let device = &self.devices[ev.u64 as usize];
        match device.read() {
          Ok(events) if !events.is_empty() => {
            // Convert raw events to the common format and invoke the callback
            for raw_event in events {
              let event: Option<InputEvent> = raw_event.into();
              if let Some(event) = event {
                // On Wayland we need to detect the global shortcuts manually
                if let InputEvent::Keyboard(key_event) = &event {
                  if let Some(hotkey) = (*hotkey_filter).process_event(key_event) {
                    event_callback(InputEvent::HotKey(HotKeyEvent { hotkey_id: hotkey }));
                  }
                }

                event_callback(event);
              } else {
                trace!("unable to convert raw event to input event");
              }
            }
          }
          Ok(_) => { /* SKIP EMPTY */ }
          Err(err) => {
            if let Some(DeviceError::FailedReadNoSuchDevice) = err.downcast_ref::<DeviceError>() {
              warn!("Can't read from device {}, this error usually means the device has been disconnected, removing from epoll.", device.get_path());

              if unsafe {
                epoll_ctl(
                  *epfd,
                  EPOLL_CTL_DEL,
                  device.get_raw_fd(),
                  std::ptr::null_mut(),
                )
              } != 0
              {
                error!(
                  "Could not remove {} from epoll, errno {}",
                  device.get_path(),
                  unsafe { *errno_ptr }
                );
                return Err(EVDEVSourceError::Internal().into());
              }
            } else {
              error!("Can't read from device {}: {}", device.get_path(), err);
            }
          }
        }
      }
    }
  }
}

#[derive(Error, Debug)]
pub enum EVDEVSourceError {
  #[error("initialization failed")]
  InitFailure(),

  #[error("permission denied")]
  PermissionDenied(),

  #[error("no devices")]
  NoDevices(),

  #[error("internal error")]
  Internal(),
}

impl From<RawInputEvent> for Option<InputEvent> {
  fn from(raw: RawInputEvent) -> Option<InputEvent> {
    match raw {
      RawInputEvent::Keyboard(keyboard_event) => {
        let (key, variant) = key_sym_to_key(keyboard_event.sym as i32);
        let value = if keyboard_event.value.is_empty() {
          None
        } else {
          Some(keyboard_event.value)
        };

        let status = if keyboard_event.state == KEY_STATE_PRESS {
          Pressed
        } else if keyboard_event.state == KEY_STATE_RELEASE {
          Released
        } else {
          // Filter out the "repeated" events
          return None;
        };

        return Some(InputEvent::Keyboard(KeyboardEvent {
          key,
          value,
          status,
          variant,
          code: keyboard_event.code,
        }));
      }
      RawInputEvent::Mouse(mouse_event) => {
        let button = raw_to_mouse_button(mouse_event.code);

        let status = if mouse_event.is_down {
          Pressed
        } else {
          Released
        };

        if let Some(button) = button {
          return Some(InputEvent::Mouse(MouseEvent { button, status }));
        }
      }
    }

    None
  }
}

// Mappings from: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values
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

    // UI
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

    // Numpad
    0xFFB0 => (Numpad0, None),
    0xFFB1 => (Numpad1, None),
    0xFFB2 => (Numpad2, None),
    0xFFB3 => (Numpad3, None),
    0xFFB4 => (Numpad4, None),
    0xFFB5 => (Numpad5, None),
    0xFFB6 => (Numpad6, None),
    0xFFB7 => (Numpad7, None),
    0xFFB8 => (Numpad8, None),
    0xFFB9 => (Numpad9, None),

    // Other keys, includes the raw code provided by the operating system
    _ => (Other(key_sym), None),
  }
}

// These codes can be found in the "input-event-codes.h" header file
fn raw_to_mouse_button(raw: u16) -> Option<MouseButton> {
  match raw {
    BTN_LEFT => Some(MouseButton::Left),
    BTN_RIGHT => Some(MouseButton::Right),
    BTN_MIDDLE => Some(MouseButton::Middle),
    BTN_SIDE => Some(MouseButton::Button1),
    BTN_EXTRA => Some(MouseButton::Button2),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use device::RawMouseEvent;

  use crate::event::{InputEvent, Key::Other, KeyboardEvent};

  use super::{
    device::{RawInputEvent, RawKeyboardEvent},
    *,
  };

  #[test]
  fn raw_to_input_event_keyboard_works_correctly() {
    let raw = RawInputEvent::Keyboard(RawKeyboardEvent {
      sym: 0x4B,
      value: "k".to_owned(),
      state: KEY_STATE_RELEASE,
      code: 0,
    });

    let result: Option<InputEvent> = raw.into();
    assert_eq!(
      result.unwrap(),
      InputEvent::Keyboard(KeyboardEvent {
        key: Other(0x4B),
        status: Released,
        value: Some("k".to_string()),
        variant: None,
        code: 0,
      })
    );
  }

  #[test]
  fn raw_to_input_event_mouse_works_correctly() {
    let raw = RawInputEvent::Mouse(RawMouseEvent {
      code: BTN_RIGHT,
      is_down: false,
    });

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
