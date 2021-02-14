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

use std::ffi::CString;

use libc::{c_uint, close, ioctl, open, O_NONBLOCK, O_WRONLY};
use scopeguard::ScopeGuard;

use anyhow::Result;
use thiserror::Error;

use super::ffi::{
  setup_uinput_device, ui_dev_create, ui_dev_destroy, ui_set_evbit, ui_set_keybit, uinput_emit,
  EV_KEY,
};

pub struct UInputDevice {
  fd: i32,
}

impl UInputDevice {
  pub fn new() -> Result<UInputDevice> {
    let uinput_path = CString::new("/dev/uinput").expect("unable to generate /dev/uinput path");
    let raw_fd = unsafe { open(uinput_path.as_ptr(), O_WRONLY | O_NONBLOCK) };
    if raw_fd < 0 {
      return Err(UInputDeviceError::OpenFailed().into());
    }
    let fd = scopeguard::guard(raw_fd, |raw_fd| unsafe {
      close(raw_fd);
    });

    // Enable keyboard events
    if unsafe { ioctl(*fd, ui_set_evbit(), EV_KEY as c_uint) } != 0 {
      return Err(UInputDeviceError::KeyEVBitFailed().into());
    }

    // Register all keycodes
    for key_code in 0..256 {
      if unsafe { ioctl(*fd, ui_set_keybit(), key_code) } != 0 {
        return Err(UInputDeviceError::KeyBitFailed().into());
      }
    }

    // Register the virtual device
    if unsafe { setup_uinput_device(*fd) } != 0 {
      return Err(UInputDeviceError::DeviceSetupFailed().into());
    }

    // Create the device
    if unsafe { ioctl(*fd, ui_dev_create()) } != 0 {
      return Err(UInputDeviceError::DeviceCreateFailed().into());
    }

    Ok(Self {
      fd: ScopeGuard::into_inner(fd),
    })
  }

  pub fn emit(&self, key_code: u32, pressed: bool) {
    let pressed = if pressed { 1 } else { 0 };
    unsafe {
      uinput_emit(self.fd, key_code, pressed);
    }
  }
}

impl Drop for UInputDevice {
  fn drop(&mut self) {
    unsafe {
      ioctl(self.fd, ui_dev_destroy());
      close(self.fd);
    }
  }
}

#[derive(Error, Debug)]
pub enum UInputDeviceError {
  #[error("could not open uinput device")]
  OpenFailed(),

  #[error("could not set keyboard evbit")]
  KeyEVBitFailed(),

  #[error("could not set keyboard keybit")]
  KeyBitFailed(),

  #[error("could not register virtual device")]
  DeviceSetupFailed(),

  #[error("could not create uinput device")]
  DeviceCreateFailed(),
}
