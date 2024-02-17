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
use log::info;

pub mod keys;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
mod x11;

#[cfg(target_os = "linux")]
mod evdev;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod mac;

pub trait Injector {
  fn send_string(&self, string: &str, options: InjectionOptions) -> Result<()>;
  fn send_keys(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()>;
  fn send_key_combination(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()>;
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct InjectionOptions {
  // Delay between injected events
  pub delay: i32,

  // Use original libxdo methods instead of patched version
  // using XSendEvent rather than XTestFakeKeyEvent
  // NOTE: Only relevant on X11 linux systems.
  pub disable_fast_inject: bool,

  // Used to set a modifier-specific delay.
  // NOTE: Only relevant on Wayland systems.
  pub evdev_modifier_delay: u32,

  // If true, use the xdotool fallback to perform the expansions.
  // NOTE: Only relevant on Linux-X11 systems.
  pub x11_use_xdotool_fallback: bool,
}

impl Default for InjectionOptions {
  fn default() -> Self {
    #[allow(clippy::if_same_then_else)]
    let default_delay = if cfg!(target_os = "windows") {
      0
    } else if cfg!(target_os = "macos") {
      1
    } else if cfg!(target_os = "linux") {
      #[allow(clippy::bool_to_int_with_if)]
      if cfg!(feature = "wayland") {
        1
      } else {
        0
      }
    } else {
      panic!("unsupported OS");
    };

    Self {
      delay: default_delay,
      disable_fast_inject: false,
      evdev_modifier_delay: 10,
      x11_use_xdotool_fallback: false,
    }
  }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct InjectorCreationOptions {
  // Only relevant in X11 Linux systems, use the EVDEV backend instead of X11.
  pub use_evdev: bool,

  // Overwrite the list of modifiers to be scanned when
  // populating the evdev injector lookup maps
  pub evdev_modifiers: Option<Vec<u32>>,

  // Overwrite the maximum number of modifiers used tested in
  // a single combination to populate the lookup maps
  pub evdev_max_modifier_combination_len: Option<i32>,

  // Can be used to overwrite the keymap configuration
  // used by espanso to inject key presses.
  pub evdev_keyboard_rmlvo: Option<KeyboardConfig>,

  // An optional provider that can be used by the injector
  // to determine which keys are pressed at the time of injection.
  // This is needed on Wayland to "wait" for key releases when
  // the injected string contains a key that it's currently pressed.
  // Otherwise, a key that is already pressed cannot be injected.
  pub keyboard_state_provider: Option<Box<dyn KeyboardStateProvider>>,
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

pub trait KeyboardStateProvider {
  fn is_key_pressed(&self, code: u32) -> bool;
}

#[cfg(target_os = "windows")]
pub fn get_injector(_options: InjectorCreationOptions) -> Result<Box<dyn Injector>> {
  info!("using Win32Injector");
  Ok(Box::new(win32::Win32Injector::new()))
}

#[cfg(target_os = "macos")]
pub fn get_injector(_options: InjectorCreationOptions) -> Result<Box<dyn Injector>> {
  info!("using MacInjector");
  Ok(Box::new(mac::MacInjector::new()))
}

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_injector(options: InjectorCreationOptions) -> Result<Box<dyn Injector>> {
  if options.use_evdev {
    info!("using EVDEVInjector");
    Ok(Box::new(evdev::EVDEVInjector::new(options)?))
  } else {
    info!("using X11ProxyInjector");
    Ok(Box::new(x11::X11ProxyInjector::new()?))
  }
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_injector(options: InjectorCreationOptions) -> Result<Box<dyn Injector>> {
  info!("using EVDEVInjector");
  Ok(Box::new(evdev::EVDEVInjector::new(options)?))
}
