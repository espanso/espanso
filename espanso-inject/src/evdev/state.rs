// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use std::ffi::CStr;

use scopeguard::ScopeGuard;

use anyhow::Result;
use thiserror::Error;

use super::{ffi::{xkb_state, xkb_state_key_get_one_sym, xkb_state_key_get_utf8, xkb_state_new, xkb_state_unref, xkb_state_update_key}, keymap::Keymap};

pub struct State {
  state: *mut xkb_state,
}

impl State {
  pub fn new(keymap: &Keymap) -> Result<State> {
    let raw_state = unsafe { xkb_state_new(keymap.get_handle()) };
    let state = scopeguard::guard(raw_state, |raw_state| unsafe {
      xkb_state_unref(raw_state);
    });

    if raw_state.is_null() {
      return Err(StateError::FailedCreation().into());
    }

    Ok(Self {
      state: ScopeGuard::into_inner(state),
    })
  }

  pub fn update_key(&self, code: u32, pressed: bool) {
    let direction = if pressed {
      super::ffi::xkb_key_direction::DOWN
    } else {
      super::ffi::xkb_key_direction::UP
    };
    unsafe {
      xkb_state_update_key(self.state, code, direction);
    }
  }

  pub fn get_string(&self, code: u32) -> Option<String> {
    let mut buffer: [u8; 16] = [0; 16];
    let len = unsafe {
      xkb_state_key_get_utf8(
        self.state,
        code,
        buffer.as_mut_ptr() as *mut i8,
        std::mem::size_of_val(&buffer),
      )
    };
    if len > 0 {
      let content_raw = unsafe { CStr::from_ptr(buffer.as_ptr() as *mut i8) };
      let string = content_raw.to_string_lossy().to_string();
      if string.is_empty() {
        None
      } else {
        Some(string)
      }
    } else {
      None
    }
  }

  pub fn get_sym(&self, code: u32) -> Option<u32> {
    let sym = unsafe { xkb_state_key_get_one_sym(self.state, code) };
    if sym == 0 {
      None
    } else {
      Some(sym)
    }
  }
}

impl Drop for State {
  fn drop(&mut self) {
    unsafe {
      xkb_state_unref(self.state);
    }
  }
}

#[derive(Error, Debug)]
pub enum StateError {
  #[error("could not create xkb state")]
  FailedCreation(),
}
