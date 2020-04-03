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

// This functions are used to check if the required dependencies and conditions are satisfied
// before starting espanso

#[cfg(target_os = "linux")]
pub fn check_preconditions() -> bool {
    use std::process::Command;

    let mut result = true;

    // Make sure notify-send is installed
    let status = Command::new("notify-send")
        .arg("-v")
        .output();
    if status.is_err() {
        println!("Error: 'notify-send' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    // Make sure xclip is installed
    let status = Command::new("xclip")
        .arg("-version")
        .output();
    if status.is_err() {
        println!("Error: 'xclip' command is needed for espanso to work correctly, please install it.");
        result = false;
    }

    result
}

#[cfg(target_os = "macos")]
pub fn check_preconditions() -> bool {
    // Make sure no app is currently using secure input.
    let secure_input_app = crate::system::macos::MacSystemManager::get_secure_input_application();

    if let Some((app_name, process)) = secure_input_app {
        eprintln!("WARNING: An application is currently using SecureInput and might prevent espanso from working correctly.");
        eprintln!();
        eprintln!("APP: {}", app_name);
        eprintln!("PROC: {}", process);
        eprintln!();
        eprintln!("Please close it or disable SecureInput for that application (most apps that use it have a");
        eprintln!("setting to disable it).");
        eprintln!("Until then, espanso might not work as expected.");
    }

    true
}

#[cfg(target_os = "windows")]
pub fn check_preconditions() -> bool {
    // Nothing needed on windows
    true
}