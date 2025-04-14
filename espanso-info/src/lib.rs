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
pub fn get_provider() -> Result<Box<dyn AppInfoProvider>> {
  info!("using Win32AppInfoProvider");
  Ok(Box::new(win32::WinAppInfoProvider::new()))
}

#[cfg(target_os = "macos")]
pub fn get_provider() -> Result<Box<dyn AppInfoProvider>> {
  info!("using CocoaAppInfoProvider");
  Ok(Box::new(cocoa::CocoaAppInfoProvider::new()))
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

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
#[cfg(target_os = "windows")]
static EXPANSION_NUM_EVENTS_REMAINING: AtomicUsize = AtomicUsize::new(0);
#[cfg(target_os = "windows")]
pub fn add_expansion_events(ev_count: usize) {
  EXPANSION_NUM_EVENTS_REMAINING.fetch_add(ev_count, SeqCst);
}
#[cfg(target_os = "windows")]
pub fn decr_expansion_events() {
  if EXPANSION_NUM_EVENTS_REMAINING.fetch_sub(1, SeqCst) == 0 {
    // Defensively do saturating subtract. Events may have been added, so can't unconditionally
    // store 0.
    let _ = EXPANSION_NUM_EVENTS_REMAINING.compare_exchange(usize::MAX, 0, SeqCst, SeqCst);
  }
}
#[cfg(target_os = "windows")]
pub fn expansion_is_in_progress() -> bool {
  EXPANSION_NUM_EVENTS_REMAINING.load(SeqCst) > 0
}
