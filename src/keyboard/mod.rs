/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait KeyboardManager {
    fn send_string(&self, s: &str);
    fn send_enter(&self);
    fn trigger_paste(&self);
    fn delete_string(&self, count: i32);
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_manager() -> impl KeyboardManager {
    windows::WindowsKeyboardManager{}
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_manager() -> impl KeyboardManager {
    linux::LinuxKeyboardManager{}
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_manager() -> impl KeyboardManager {
    macos::MacKeyboardManager{}
}