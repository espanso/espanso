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
pub(crate) mod macos;

use std::sync::mpsc::Sender;
use crate::event::Event;
use std::path::PathBuf;
use std::fs::create_dir_all;

pub trait Context {
    fn eventloop(&self);
}

pub fn get_data_dir() -> PathBuf {
    let data_dir = dirs::data_dir().expect("Can't obtain data_dir(), terminating.");
    let espanso_dir = data_dir.join("espanso");
    create_dir_all(&espanso_dir).expect("Error creating espanso data directory");
    espanso_dir
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    macos::MacContext::new(send_channel)
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    linux::LinuxContext::new(send_channel)
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    windows::WindowsContext::new(send_channel)
}