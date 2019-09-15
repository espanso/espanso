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

pub trait UIManager {
    fn notify(&self, message: &str);
    fn show_menu(&self, menu: Vec<MenuItem>);
}

pub enum MenuItemType {
    Button,
    Separator,
}

pub struct MenuItem {
    pub item_id: i32,
    pub item_type: MenuItemType,
    pub item_name: String,
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_uimanager() -> impl UIManager {
    macos::MacUIManager::new()
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_uimanager() -> impl UIManager {
    linux::LinuxUIManager::new()
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_uimanager() -> impl UIManager {
    windows::WindowsUIManager::new()
}