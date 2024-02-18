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

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::add_espanso_to_path;
#[cfg(target_os = "macos")]
pub use macos::is_espanso_in_path;
#[cfg(target_os = "macos")]
pub use macos::remove_espanso_from_path;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use win::add_espanso_to_path;
#[cfg(target_os = "windows")]
pub use win::is_espanso_in_path;
#[cfg(target_os = "windows")]
pub use win::remove_espanso_from_path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::add_espanso_to_path;
#[cfg(target_os = "linux")]
pub use linux::is_espanso_in_path;
#[cfg(target_os = "linux")]
pub use linux::remove_espanso_from_path;
