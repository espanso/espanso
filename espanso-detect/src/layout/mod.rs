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

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
mod x11;

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
mod gnome;

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
mod wayland;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_active_layout() -> Option<String> {
  x11::get_active_layout()
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_active_layout() -> Option<String> {
  if gnome::is_gnome() {
    gnome::get_active_layout()
  } else {
    use log::debug;
    debug!(
      "Wayland compositor detected: {}",
      wayland::get_compositor_name()
    );
    None
  }
}

#[cfg(not(target_os = "linux"))]
pub fn get_active_layout() -> Option<String> {
  // Not available on Windows and macOS yet
  None
}
