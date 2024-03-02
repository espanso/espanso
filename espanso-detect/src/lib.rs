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
use hotkey::HotKey;
use log::info;

pub mod event;
pub mod hotkey;
pub mod layout;

#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub mod x11;

#[cfg(target_os = "linux")]
pub mod evdev;

#[cfg(target_os = "macos")]
pub mod mac;

pub type SourceCallback = Box<dyn Fn(event::InputEvent)>;

pub trait Source {
  fn initialize(&mut self) -> Result<()>;
  fn eventloop(&self, event_callback: SourceCallback) -> Result<()>;
}

#[allow(dead_code)]
pub struct SourceCreationOptions {
  // Only relevant in X11 Linux systems, use the EVDEV backend instead of X11.
  pub use_evdev: bool,

  // Can be used to overwrite the keymap configuration
  // used by espanso to inject key presses.
  pub evdev_keyboard_rmlvo: Option<KeyboardConfig>,

  // List of global hotkeys the detection module has to register
  // NOTE: Hotkeys don't work under the EVDEV backend yet (Wayland)
  pub hotkeys: Vec<HotKey>,

  // If true, filter out keyboard events without an explicit HID device source on Windows.
  // This is needed to filter out the software-generated events, including
  // those from espanso, but might need to be disabled when using some software-level keyboards.
  // Disabling this option might conflict with the undo feature.
  pub win32_exclude_orphan_events: bool,

  // The maximum interval (in milliseconds) for which a keyboard layout
  // can be cached. If switching often between different layouts, you
  // could lower this amount to avoid the "lost detection" effect described
  // in this issue: https://github.com/espanso/espanso/issues/745
  pub win32_keyboard_layout_cache_interval: i64,
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
      hotkeys: Vec::new(),
      win32_exclude_orphan_events: true,
      win32_keyboard_layout_cache_interval: 2000,
    }
  }
}

#[cfg(target_os = "windows")]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using Win32Source");
  Ok(Box::new(win32::Win32Source::new(
    &options.hotkeys,
    options.win32_exclude_orphan_events,
    options.win32_keyboard_layout_cache_interval,
  )))
}

#[cfg(target_os = "macos")]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using CocoaSource");
  Ok(Box::new(mac::CocoaSource::new(&options.hotkeys)))
}

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  if options.use_evdev {
    info!("using EVDEVSource");
    Ok(Box::new(evdev::EVDEVSource::new(options)))
  } else {
    info!("using X11Source");
    Ok(Box::new(x11::X11Source::new(&options.hotkeys)))
  }
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_source(options: SourceCreationOptions) -> Result<Box<dyn Source>> {
  info!("using EVDEVSource");
  Ok(Box::new(evdev::EVDEVSource::new(options)))
}

pub use layout::get_active_layout;
