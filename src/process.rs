/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use log::warn;
use std::io;
use std::process::{Child, Command, Stdio};

#[cfg(target_os = "windows")]
pub fn spawn_process(cmd: &str, args: &Vec<String>) -> io::Result<Child> {
    use std::os::windows::process::CommandExt;
    Command::new(cmd)
        .creation_flags(0x08000008) // Detached Process without window
        .args(args)
        .spawn()
}

#[cfg(not(target_os = "windows"))]
pub fn spawn_process(cmd: &str, args: &Vec<String>) -> io::Result<Child> {
    Command::new(cmd).args(args).spawn()
}
