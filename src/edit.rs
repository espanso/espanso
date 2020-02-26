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

use crate::config::ConfigSet;

#[cfg(target_os = "linux")]
pub fn open_editor(config: &ConfigSet) -> bool {
    // TODO
}

#[cfg(target_os = "macos")]
pub fn open_editor(config: &ConfigSet) -> bool {
    // TODO
}

#[cfg(target_os = "windows")]
pub fn open_editor(config: &ConfigSet) -> bool {
    use std::process::Command;

    // Get the configuration file path
    let file_path = crate::context::get_config_dir().join(crate::config::DEFAULT_CONFIG_FILE_NAME);

    // Start the editor and wait for its termination
    let status = Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("/wait")
        .arg("C:\\Windows\\System32\\notepad.exe")
        .arg(file_path)
        .spawn();

    if let Ok(mut child) = status {
        // Wait for the user to edit the configuration
        child.wait();

        // TODO: instead of waiting, a file watcher should be started to detect file changes and
        // after each of them a reload should be issued

        println!("Ok");
        true
    }else{
        println!("Error: could not start editor.");
        false
    }
}