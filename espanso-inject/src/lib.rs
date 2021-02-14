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

pub mod keys;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "linux")]
mod evdev;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod mac;

#[macro_use]
extern crate lazy_static;

pub trait Injector {
  fn send_string(&self, string: &str, options: InjectionOptions) -> Result<()>;
  fn send_keys(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()>;
  fn send_key_combination(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()>;
}

#[allow(dead_code)]
pub struct InjectionOptions {
  // Delay between injected events
  delay: i32,

  // Use original libxdo methods instead of patched version
  // using XSendEvent rather than XTestFakeKeyEvent
  // NOTE: Only relevant on X11 linux systems.
  disable_fast_inject: bool,
}

impl Default for InjectionOptions {
  fn default() -> Self {
    let default_delay = if cfg!(target_os = "windows") {
      0
    } else if cfg!(target_os = "macos") {
      2
    } else if cfg!(target_os = "linux") {
      0
    } else {
      panic!("unsupported OS");
    };

    Self {
      delay: default_delay,
      disable_fast_inject: false,
    }
  }
}

#[allow(dead_code)]
pub struct InjectorCreationOptions {
  // Only relevant in Linux systems
  use_evdev: bool,

  // Overwrite the list of modifiers to be scanned when 
  // populating the evdev injector lookup maps
  evdev_modifiers: Option<Vec<u32>>,

  // Overwrite the maximum number of modifiers used tested in
  // a single combination to populate the lookup maps
  evdev_max_modifier_combination_len: Option<i32>,

  // Can be used to overwrite the keymap configuration
  // used by espanso to inject key presses.
  evdev_keyboard_rmlvo: Option<KeyboardConfig>,
}

// This struct identifies the keyboard layout that
// should be used by EVDEV when loading the keymap.
// For more information: https://xkbcommon.org/doc/current/structxkb__rule__names.html
pub struct KeyboardConfig {
  pub rules: Option<String>,
  pub model: Option<String>,
  pub layout: Option<String>,
  pub variant: Option<String>,
  pub options: Option<String>,
}

impl Default for InjectorCreationOptions {
  fn default() -> Self {
    Self {
      use_evdev: false,
      evdev_modifiers: None,
      evdev_max_modifier_combination_len: None,
      evdev_keyboard_rmlvo: None,
    }
  }
}

#[cfg(target_os = "windows")]
pub fn get_injector(_options: InjectorOptions) -> impl Injector {
  win32::Win32Injector::new()
}

#[cfg(target_os = "macos")]
pub fn get_injector(_options: InjectorOptions) -> impl Injector {
  mac::MacInjector::new()
}

#[cfg(target_os = "linux")]
pub fn get_injector(options: InjectorCreationOptions) -> Result<impl Injector> {
  // TODO: differenciate based on the options
  //x11::X11Injector::new()
  evdev::EVDEVInjector::new(options)
}

