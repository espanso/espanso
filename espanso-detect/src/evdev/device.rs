// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use anyhow::Result;
use libc::{input_event, size_t, ssize_t, EWOULDBLOCK, O_CLOEXEC, O_NONBLOCK, O_RDONLY};
use log::{trace};
use scopeguard::ScopeGuard;
use std::os::unix::io::AsRawFd;
use std::{
  ffi::{c_void, CStr},
  fs::OpenOptions,
  mem::zeroed,
};
use std::{fs::File, os::unix::fs::OpenOptionsExt};
use thiserror::Error;

use super::{
  ffi::{
    is_keyboard, xkb_key_direction, xkb_keycode_t, xkb_keymap_key_repeats,
    xkb_state, xkb_state_get_keymap, xkb_state_key_get_one_sym,
    xkb_state_key_get_utf8, xkb_state_new, xkb_state_unref,
    xkb_state_update_key, EV_KEY,
  },
  keymap::Keymap,
};

const EVDEV_OFFSET: i32 = 8;
const KEY_STATE_RELEASE: i32 = 0;
const KEY_STATE_PRESS: i32 = 1;
const KEY_STATE_REPEAT: i32 = 2;

#[derive(Debug)]
pub enum RawInputEvent {
  Keyboard(KeyboardEvent),
  Mouse(MouseEvent),
}

#[derive(Debug)]
pub struct KeyboardEvent {
  pub sym: u32,
  pub value: String,
  pub is_down: bool,
}

#[derive(Debug)]
pub struct MouseEvent {
  pub code: u16,
  pub is_down: bool,
}

pub struct Device {
  path: String,
  file: File,
  state: *mut xkb_state,
}

impl Device {
  pub fn from(path: &str, keymap: &Keymap) -> Result<Device> {
    let file = OpenOptions::new()
      .read(true)
      .custom_flags(O_NONBLOCK | O_CLOEXEC | O_RDONLY)
      .open(&path)?;

    if unsafe { is_keyboard(file.as_raw_fd()) == 0 } {
      return Err(DeviceError::InvalidDevice(path.to_string()).into());
    }

    let raw_state = unsafe { xkb_state_new(keymap.get_handle()) };
    // Automatically close the state if the function does not return correctly
    let state = scopeguard::guard(raw_state, |raw_state| {
      unsafe {
        xkb_state_unref(raw_state);
      }
    });

    if raw_state.is_null() {
      return Err(DeviceError::InvalidState(path.to_string()).into());
    }

    Ok(Self {
      path: path.to_string(),
      file,
      // Release the state without freeing it
      state: ScopeGuard::into_inner(state),
    })
  }

  pub fn get_state(&self) -> *mut xkb_state {
    self.state
  }

  pub fn get_raw_fd(&self) -> i32 {
    self.file.as_raw_fd()
  }

  pub fn get_path(&self) -> String {
    self.path.to_string()
  }

  pub fn read(&self) -> Result<Vec<RawInputEvent>> {
    let errno_ptr = unsafe { libc::__errno_location() };
    let mut len: ssize_t;
    let mut evs: [input_event; 16] = unsafe { std::mem::zeroed() };
    let mut events = Vec::new();

    loop {
      len = unsafe {
        libc::read(
          self.file.as_raw_fd(),
          evs.as_mut_ptr() as *mut c_void,
          std::mem::size_of_val(&evs),
        )
      };
      if len <= 0 {
        break;
      }

      let nevs: size_t = len as usize / std::mem::size_of::<input_event>();
      
      #[allow(clippy::needless_range_loop)]
      for i in 0..nevs {
        let event = self.process_event(evs[i].type_, evs[i].code, evs[i].value);
        if let Some(event) = event {
          events.push(event);
        }
      }
    }

    if len < 0 && unsafe { *errno_ptr } != EWOULDBLOCK {
      return Err(DeviceError::BlockingReadOperation().into());
    }

    Ok(events)
  }

  fn process_event(&self, _type: u16, code: u16, value: i32) -> Option<RawInputEvent> {
    if _type != EV_KEY {
      return None;
    }

    let is_down = value == KEY_STATE_PRESS;

    // Check if the current event originated from a mouse
    if code >= 0x110 && code <= 0x117 {
      // Mouse event
      return Some(RawInputEvent::Mouse(MouseEvent { code, is_down }));
    }

    // Keyboard event

    let keycode: xkb_keycode_t = EVDEV_OFFSET as u32 + code as u32;
    let keymap = unsafe { xkb_state_get_keymap(self.get_state()) };

    if value == KEY_STATE_REPEAT && unsafe { xkb_keymap_key_repeats(keymap, keycode) } != 0 {
      return None;
    }

    let sym = unsafe { xkb_state_key_get_one_sym(self.get_state(), keycode) };
    if sym == 0 {
      return None;
    }

    // Extract the utf8 char
    let mut buffer: [u8; 16] = [0; 16];
    unsafe {
      xkb_state_key_get_utf8(
        self.get_state(),
        keycode,
        buffer.as_mut_ptr() as *mut i8,
        std::mem::size_of_val(&buffer),
      )
    };
    let content_raw = unsafe { CStr::from_ptr(buffer.as_ptr() as *mut i8) };
    let content = content_raw.to_string_lossy().to_string();

    let event = KeyboardEvent {
      is_down,
      sym,
      value: content,
    };

    if value == KEY_STATE_RELEASE {
      unsafe { xkb_state_update_key(self.get_state(), keycode, xkb_key_direction::UP) };
    } else {
      unsafe { xkb_state_update_key(self.get_state(), keycode, xkb_key_direction::DOWN) };
    }

    Some(RawInputEvent::Keyboard(event))
  }
}

impl Drop for Device {
  fn drop(&mut self) {
    unsafe {
      xkb_state_unref(self.state);
    }
  }
}

pub fn get_devices(keymap: &Keymap) -> Result<Vec<Device>> {
  let mut keyboards = Vec::new();
  let dirs = std::fs::read_dir("/dev/input/")?;
  for entry in dirs {
    match entry {
      Ok(device) => {
        // Skip non-eventX devices
        if !device.file_name().to_string_lossy().starts_with("event") {
          continue;
        }

        let path = device.path().to_string_lossy().to_string();
        let keyboard = Device::from(&path, keymap);
        match keyboard {
          Ok(keyboard) => {
            keyboards.push(keyboard);
          }
          Err(error) => {
            trace!("error opening keyboard: {}", error);
          }
        }
      }
      Err(error) => {
        trace!("could not read keyboard device: {}", error);
      }
    }
  }

  if keyboards.is_empty() {
    return Err(DeviceError::NoDevicesFound().into());
  }

  Ok(keyboards)
}

#[derive(Error, Debug)]
pub enum DeviceError {
  #[error("could not create xkb state for `{0}`")]
  InvalidState(String),

  #[error("`{0}` is not a valid device")]
  InvalidDevice(String),

  #[error("no devices found")]
  NoDevicesFound(),

  #[error("read operation can't block device")]
  BlockingReadOperation(),
}
