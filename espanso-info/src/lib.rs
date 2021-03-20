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

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
mod x11;

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
mod wayland;

#[cfg(target_os = "macos")]
mod cocoa;

pub trait AppInfoProvider {
  fn get_info(&self) -> AppInfo;
}

#[derive(Debug, Clone)]
pub struct AppInfo {
  pub title: Option<String>,
  pub exec: Option<String>,
  pub class: Option<String>,  
}

#[cfg(target_os = "windows")]
pub fn get_clipboard(_: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
  info!("using Win32Clipboard");
  Ok(Box::new(win32::Win32Clipboard::new()?))
}

#[cfg(target_os = "macos")]
pub fn get_clipboard(_: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
  info!("using CocoaClipboard");
  Ok(Box::new(cocoa::CocoaClipboard::new()?))
}

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_provider() -> Result<Box<dyn AppInfoProvider>> {
  info!("using X11AppInfoProvider");
  Ok(Box::new(x11::X11AppInfoProvider::new()))
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_provider() -> Result<Box<dyn AppInfoProvider>> {
  info!("using WaylandAppInfoProvider");
  Ok(Box::new(wayland::WaylandAppInfoProvider::new()))
}