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

// This functions are used to check if the required dependencies are satisfied
// before starting espanso

#[cfg(target_os = "linux")]
pub fn check_dependencies() -> bool {
    use std::process::Command;

    let mut result = true;

    // Make sure notify-send is installed
    let status = Command::new("notify-send")
        .arg("-v")
        .output();
    if let Err(_) = status {
        println!("Error: 'notify-send' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    // Make sure xclip is installed
    let status = Command::new("xclip")
        .arg("-version")
        .output();
    if let Err(_) = status {
        println!("Error: 'xclip' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    result
}

#[cfg(target_os = "macos")]
pub fn check_dependencies() -> bool {
    // Nothing to do here
    true
}

#[cfg(target_os = "windows")]
pub fn check_dependencies() -> bool {
    // Nothing needed on windows
    true
}