/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2022 Federico Terzi
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

use std::{convert::TryInto, ffi::CString};

use crate::Injector;
use anyhow::{bail, Context, Result};
use libc::c_int;
use log::debug;

mod ffi;
use self::ffi::{xdo_t, CURRENTWINDOW};

use super::ffi::{Window, XGetInputFocus, XQueryKeymap, XTestFakeKeyEvent};

pub struct X11XDOToolInjector {
  xdo: *const xdo_t,
}

impl X11XDOToolInjector {
  pub fn new() -> Result<Self> {
    let xdo = unsafe { ffi::xdo_new(std::ptr::null()) };
    if xdo.is_null() {
      bail!("unable to initialize xdo_t instance");
    }

    debug!("initialized xdo_t object");

    Ok(Self { xdo })
  }

  fn xfake_release_all_keys(&self) {
    let mut keys: [u8; 32] = [0; 32];
    unsafe {
      XQueryKeymap((*self.xdo).xdpy, keys.as_mut_ptr());
    }

    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
      // Only those that are pressed should be changed
      if keys[i] != 0 {
        for k in 0..8 {
          if (keys[i] & (1 << k)) != 0 {
            let key_code = i * 8 + k;
            unsafe {
              XTestFakeKeyEvent((*self.xdo).xdpy, key_code as u32, 0, 0);
            }
          }
        }
      }
    }
  }

  fn xfake_send_string(
    &self,
    string: &str,
    options: crate::InjectionOptions,
  ) -> anyhow::Result<()> {
    // It may happen that when an expansion is triggered, some keys are still pressed.
    // This causes a problem if the expanded match contains that character, as the injection
    // will not be able to register that keypress (as it is already pressed).
    // To solve the problem, before an expansion we get which keys are currently pressed
    // and inject a key_release event so that they can be further registered.
    self.xfake_release_all_keys();

    let c_string = CString::new(string).context("unable to create CString")?;
    let delay = options.delay * 1000;

    unsafe {
      ffi::xdo_enter_text_window(
        self.xdo,
        CURRENTWINDOW,
        c_string.as_ptr(),
        delay.try_into().unwrap(),
      );
    }

    Ok(())
  }

  fn fast_release_all_keys(&self) {
    let mut focused: Window = 0;
    let mut revert_to: c_int = 0;
    unsafe {
      XGetInputFocus((*self.xdo).xdpy, &mut focused, &mut revert_to);
    }

    let mut keys: [u8; 32] = [0; 32];
    unsafe {
      XQueryKeymap((*self.xdo).xdpy, keys.as_mut_ptr());
    }

    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
      // Only those that are pressed should be changed
      if keys[i] != 0 {
        for k in 0..8 {
          if (keys[i] & (1 << k)) != 0 {
            let key_code = i * 8 + k;
            unsafe {
              ffi::fast_send_event(self.xdo, focused, key_code.try_into().unwrap(), 0);
            }
          }
        }
      }
    }
  }

  fn fast_send_string(&self, string: &str, options: crate::InjectionOptions) -> anyhow::Result<()> {
    // It may happen that when an expansion is triggered, some keys are still pressed.
    // This causes a problem if the expanded match contains that character, as the injection
    // will not be able to register that keypress (as it is already pressed).
    // To solve the problem, before an expansion we get which keys are currently pressed
    // and inject a key_release event so that they can be further registered.
    self.fast_release_all_keys();

    let mut focused: Window = 0;
    let mut revert_to: c_int = 0;
    unsafe {
      XGetInputFocus((*self.xdo).xdpy, &mut focused, &mut revert_to);
    }

    let c_string = CString::new(string).context("unable to create CString")?;
    let delay = options.delay * 1000;

    unsafe {
      ffi::fast_enter_text_window(
        self.xdo,
        focused,
        c_string.as_ptr(),
        delay.try_into().unwrap(),
      );
    }

    Ok(())
  }
}

impl Injector for X11XDOToolInjector {
  fn send_string(&self, string: &str, options: crate::InjectionOptions) -> anyhow::Result<()> {
    if options.disable_fast_inject {
      self.xfake_send_string(string, options)
    } else {
      self.fast_send_string(string, options)
    }
  }

  fn send_keys(
    &self,
    keys: &[crate::keys::Key],
    options: crate::InjectionOptions,
  ) -> anyhow::Result<()> {
    //todo!()
    Ok(())
  }

  fn send_key_combination(
    &self,
    keys: &[crate::keys::Key],
    options: crate::InjectionOptions,
  ) -> anyhow::Result<()> {
    //todo!()
    Ok(())
  }
}

impl Drop for X11XDOToolInjector {
  fn drop(&mut self) {
    unsafe { ffi::xdo_free(self.xdo) }
  }
}
