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

use std::path::Path;

use anyhow::Result;
use log::info;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
mod x11;

//#[cfg(target_os = "linux")]
//mod evdev;

#[cfg(target_os = "macos")]
mod mac;

#[macro_use]
extern crate lazy_static;

pub trait Clipboard {
  fn get_text(&self) -> Option<String>;
  fn set_text(&self, text: &str) -> Result<()>;
  fn set_image(&self, image_path: &Path) -> Result<()>;
  fn set_html(&self, html: &str, fallback_text: Option<&str>) -> Result<()>;
}

#[allow(dead_code)]
pub struct ClipboardOptions {
}

impl Default for ClipboardOptions {
  fn default() -> Self {
    Self {}
  }
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
pub fn get_clipboard(options: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
  info!("using X11NativeClipboard");
  Ok(Box::new(x11::native::X11NativeClipboard::new()?))
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_injector(options: InjectorCreationOptions) -> Result<Box<dyn Injector>> {
  info!("using EVDEVInjector");
  Ok(Box::new(evdev::EVDEVInjector::new(options)?))
}
