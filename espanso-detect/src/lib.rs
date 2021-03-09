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

pub mod event;

#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub mod x11;

#[cfg(target_os = "linux")]
pub mod evdev;

#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate lazy_static;

pub type SourceCallback = Box<dyn Fn(event::InputEvent)>;

pub trait Source {
  fn initialize(&mut self) -> Result<()>;
  fn eventloop(&self, event_callback: SourceCallback) -> Result<()>;
}

#[allow(dead_code)]
pub struct SourceCreationOptions {
  // Only relevant in X11 Linux systems, use the EVDEV backend instead of X11.
  use_evdev: bool,

  // Can be used to overwrite the keymap configuration
  // used by espanso to inject key presses.
  evdev_keyboard_rmlvo: Option<KeyboardConfig>,
}

// This struct identifies the keyboard layout that
// should be used by EVDEV when loading the keymap.
// For more information: https://xkbcommon.org/doc/current/structxkb__rule__names.html
#[derive(Debug, Clone)]
pub struct KeyboardConfig {
  pub rules: Option<String>,
  pub model: Option<String>,
  pub layout: Option<String>,
  pub variant: Option<String>,
  pub options: Option<String>,
}

impl Default for SourceCreationOptions {
  fn default() -> Self {
    Self {
      use_evdev: false,
      evdev_keyboard_rmlvo: None,
    }
  }
}

#[cfg(target_os = "windows")]
pub fn get_source(_options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using Win32Source");
  Ok(Box::new(win32::Win32Source::new()))
}

#[cfg(target_os = "macos")]
pub fn get_source(_options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using CocoaSource");
  Ok(Box::new(mac::CocoaSource::new()))
}

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  if options.use_evdev {
    info!("using EVDEVSource");
    Ok(Box::new(evdev::EVDEVSource::new(options)))
  } else {
    info!("using X11Source");
    Ok(Box::new(x11::X11Source::new()))
  }
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using EVDEVSource");
  Ok(Box::new(evdev::EVDEVSource::new(options)))
}
