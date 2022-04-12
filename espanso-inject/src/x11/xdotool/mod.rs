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

use std::{
  convert::TryInto,
  ffi::{CStr, CString},
};

use crate::Injector;
use anyhow::{bail, Context, Result};
use log::debug;

mod ffi;
use self::ffi::{fast_send_keysequence_window, xdo_send_keysequence_window, xdo_t, CURRENTWINDOW};

use super::ffi::{
  Display, Window, XGetInputFocus, XKeycodeToKeysym, XKeysymToString, XQueryKeymap,
  XTestFakeKeyEvent,
};

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

  fn get_focused_window(&self) -> Window {
    let mut focused_window: Window = 0;
    let mut revert_to = 0;
    unsafe {
      XGetInputFocus((*self.xdo).xdpy, &mut focused_window, &mut revert_to);
    }
    focused_window
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
    let mut keys: [u8; 32] = [0; 32];
    unsafe {
      XQueryKeymap((*self.xdo).xdpy, keys.as_mut_ptr());
    }

    let focused_window = self.get_focused_window();

    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
      // Only those that are pressed should be changed
      if keys[i] != 0 {
        for k in 0..8 {
          if (keys[i] & (1 << k)) != 0 {
            let key_code = i * 8 + k;
            unsafe {
              ffi::fast_send_event(self.xdo, focused_window, key_code.try_into().unwrap(), 0);
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

    let c_string = CString::new(string).context("unable to create CString")?;
    let delay = options.delay * 1000;

    unsafe {
      ffi::fast_enter_text_window(
        self.xdo,
        self.get_focused_window(),
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
    let key_syms: Vec<String> = keys
      .iter()
      .filter_map(|key| unsafe { convert_key_to_keysym((*self.xdo).xdpy, key) })
      .collect();

    let delay = options.delay * 1000;

    for key in key_syms {
      let c_str = CString::new(key).context("unable to generate CString")?;

      if options.disable_fast_inject {
        unsafe {
          xdo_send_keysequence_window(
            self.xdo,
            CURRENTWINDOW,
            c_str.as_ptr(),
            delay.try_into().unwrap(),
          );
        }
      } else {
        unsafe {
          fast_send_keysequence_window(
            self.xdo,
            self.get_focused_window(),
            c_str.as_ptr(),
            delay.try_into().unwrap(),
          );
        }
      }
    }

    Ok(())
  }

  fn send_key_combination(
    &self,
    keys: &[crate::keys::Key],
    options: crate::InjectionOptions,
  ) -> anyhow::Result<()> {
    let key_syms: Vec<String> = keys
      .iter()
      .filter_map(|key| unsafe { convert_key_to_keysym((*self.xdo).xdpy, key) })
      .collect();
    let key_combination = key_syms.join("+");

    let delay = options.delay * 1000;

    let c_key_combination = CString::new(key_combination).context("unable to generate CString")?;

    if options.disable_fast_inject {
      unsafe {
        xdo_send_keysequence_window(
          self.xdo,
          CURRENTWINDOW,
          c_key_combination.as_ptr(),
          delay.try_into().unwrap(),
        );
      }
    } else {
      unsafe {
        fast_send_keysequence_window(
          self.xdo,
          self.get_focused_window(),
          c_key_combination.as_ptr(),
          delay.try_into().unwrap(),
        );
      }
    }

    Ok(())
  }
}

impl Drop for X11XDOToolInjector {
  fn drop(&mut self) {
    unsafe { ffi::xdo_free(self.xdo) }
  }
}

fn convert_key_to_keysym(display: *mut Display, key: &crate::keys::Key) -> Option<String> {
  match key {
    crate::keys::Key::Alt => Some("Alt_L".to_string()),
    crate::keys::Key::CapsLock => Some("Caps_Lock".to_string()),
    crate::keys::Key::Control => Some("Control_L".to_string()),
    crate::keys::Key::Meta => Some("Meta_L".to_string()),
    crate::keys::Key::NumLock => Some("Num_Lock".to_string()),
    crate::keys::Key::Shift => Some("Shift_L".to_string()),
    crate::keys::Key::Enter => Some("Return".to_string()),
    crate::keys::Key::Tab => Some("Tab".to_string()),
    crate::keys::Key::Space => Some("space".to_string()),
    crate::keys::Key::ArrowDown => Some("downarrow".to_string()),
    crate::keys::Key::ArrowLeft => Some("leftarrow".to_string()),
    crate::keys::Key::ArrowRight => Some("rightarrow".to_string()),
    crate::keys::Key::ArrowUp => Some("uparrow".to_string()),
    crate::keys::Key::End => Some("End".to_string()),
    crate::keys::Key::Home => Some("Home".to_string()),
    crate::keys::Key::PageDown => Some("Page_Down".to_string()),
    crate::keys::Key::PageUp => Some("Page_Up".to_string()),
    crate::keys::Key::Escape => Some("Escape".to_string()),
    crate::keys::Key::Backspace => Some("BackSpace".to_string()),
    crate::keys::Key::Insert => Some("Insert".to_string()),
    crate::keys::Key::Delete => Some("Delete".to_string()),
    crate::keys::Key::F1 => Some("F1".to_string()),
    crate::keys::Key::F2 => Some("F2".to_string()),
    crate::keys::Key::F3 => Some("F3".to_string()),
    crate::keys::Key::F4 => Some("F4".to_string()),
    crate::keys::Key::F5 => Some("F5".to_string()),
    crate::keys::Key::F6 => Some("F6".to_string()),
    crate::keys::Key::F7 => Some("F7".to_string()),
    crate::keys::Key::F8 => Some("F8".to_string()),
    crate::keys::Key::F9 => Some("F9".to_string()),
    crate::keys::Key::F10 => Some("F10".to_string()),
    crate::keys::Key::F11 => Some("F11".to_string()),
    crate::keys::Key::F12 => Some("F12".to_string()),
    crate::keys::Key::F13 => Some("F13".to_string()),
    crate::keys::Key::F14 => Some("F14".to_string()),
    crate::keys::Key::F15 => Some("F15".to_string()),
    crate::keys::Key::F16 => Some("F16".to_string()),
    crate::keys::Key::F17 => Some("F17".to_string()),
    crate::keys::Key::F18 => Some("F18".to_string()),
    crate::keys::Key::F19 => Some("F19".to_string()),
    crate::keys::Key::F20 => Some("F20".to_string()),
    crate::keys::Key::A => Some("a".to_string()),
    crate::keys::Key::B => Some("b".to_string()),
    crate::keys::Key::C => Some("c".to_string()),
    crate::keys::Key::D => Some("d".to_string()),
    crate::keys::Key::E => Some("e".to_string()),
    crate::keys::Key::F => Some("f".to_string()),
    crate::keys::Key::G => Some("g".to_string()),
    crate::keys::Key::H => Some("h".to_string()),
    crate::keys::Key::I => Some("i".to_string()),
    crate::keys::Key::J => Some("j".to_string()),
    crate::keys::Key::K => Some("k".to_string()),
    crate::keys::Key::L => Some("l".to_string()),
    crate::keys::Key::M => Some("m".to_string()),
    crate::keys::Key::N => Some("n".to_string()),
    crate::keys::Key::O => Some("o".to_string()),
    crate::keys::Key::P => Some("p".to_string()),
    crate::keys::Key::Q => Some("q".to_string()),
    crate::keys::Key::R => Some("r".to_string()),
    crate::keys::Key::S => Some("s".to_string()),
    crate::keys::Key::T => Some("t".to_string()),
    crate::keys::Key::U => Some("u".to_string()),
    crate::keys::Key::V => Some("v".to_string()),
    crate::keys::Key::W => Some("w".to_string()),
    crate::keys::Key::X => Some("x".to_string()),
    crate::keys::Key::Y => Some("y".to_string()),
    crate::keys::Key::Z => Some("z".to_string()),
    crate::keys::Key::N0 => Some("0".to_string()),
    crate::keys::Key::N1 => Some("1".to_string()),
    crate::keys::Key::N2 => Some("2".to_string()),
    crate::keys::Key::N3 => Some("3".to_string()),
    crate::keys::Key::N4 => Some("4".to_string()),
    crate::keys::Key::N5 => Some("5".to_string()),
    crate::keys::Key::N6 => Some("6".to_string()),
    crate::keys::Key::N7 => Some("7".to_string()),
    crate::keys::Key::N8 => Some("8".to_string()),
    crate::keys::Key::N9 => Some("9".to_string()),
    crate::keys::Key::Numpad0 => Some("KP_0".to_string()),
    crate::keys::Key::Numpad1 => Some("KP_1".to_string()),
    crate::keys::Key::Numpad2 => Some("KP_2".to_string()),
    crate::keys::Key::Numpad3 => Some("KP_3".to_string()),
    crate::keys::Key::Numpad4 => Some("KP_4".to_string()),
    crate::keys::Key::Numpad5 => Some("KP_5".to_string()),
    crate::keys::Key::Numpad6 => Some("KP_6".to_string()),
    crate::keys::Key::Numpad7 => Some("KP_7".to_string()),
    crate::keys::Key::Numpad8 => Some("KP_8".to_string()),
    crate::keys::Key::Numpad9 => Some("KP_9".to_string()),
    crate::keys::Key::Raw(key_code) => unsafe {
      let key_sym = XKeycodeToKeysym(display, (*key_code).try_into().unwrap(), 0);
      let string = XKeysymToString(key_sym);
      let c_str = CStr::from_ptr(string);
      Some(c_str.to_string_lossy().to_string())
    },
  }
}
