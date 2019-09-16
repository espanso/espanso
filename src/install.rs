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

// This functions are used to register/unregister espanso from the system daemon manager.

use crate::config::ConfigSet;
use std::fs::create_dir_all;
use std::process::{Command, ExitStatus};

// INSTALLATION

#[cfg(target_os = "linux")]
pub fn install(config_set: ConfigSet) {
    // TODO
}

#[cfg(target_os = "macos")]
const MAC_PLIST_CONTENT : &str = include_str!("res/mac/com.federicoterzi.espanso.plist");
#[cfg(target_os = "macos")]
const MAC_PLIST_FILENAME : &str = "com.federicoterzi.espanso.plist";

#[cfg(target_os = "macos")]
pub fn install(config_set: ConfigSet) {
    let home_dir = dirs::home_dir().expect("Could not get user home directory");
    let library_dir = home_dir.join("Library");
    let agents_dir = library_dir.join("LaunchAgents");

    // Make sure agents directory exists
    if !agents_dir.exists() {
        create_dir_all(agents_dir.clone()).expect("Could not create LaunchAgents directory");
    }

    let plist_file = agents_dir.join(MAC_PLIST_FILENAME);
    if !plist_file.exists() {
        println!("Creating LaunchAgents entry: {}", plist_file.to_str().unwrap_or_default());

        let espanso_path = std::env::current_exe().expect("Could not get espanso executable path");
        println!("Entry will point to: {}", espanso_path.to_str().unwrap_or_default());

        let plist_content = String::from(MAC_PLIST_CONTENT)
            .replace("{{{espanso_path}}}", espanso_path.to_str().unwrap_or_default());

        std::fs::write(plist_file.clone(), plist_content).expect("Unable to write plist file");

        println!("Entry created correctly!")
    }

    println!("Reloading entry...");

    let res = Command::new("launchctl")
    .args(&["unload", "-w", plist_file.to_str().unwrap_or_default()])
    .output();

    let res = Command::new("launchctl")
        .args(&["load", "-w", plist_file.to_str().unwrap_or_default()])
        .status();

    if let Ok(status) = res {
        if status.success() {
            println!("Entry loaded correctly!")
        }
    }else{
        println!("Error loading new entry");
    }
}

#[cfg(target_os = "macos")]
pub fn uninstall(config_set: ConfigSet) {
    let home_dir = dirs::home_dir().expect("Could not get user home directory");
    let library_dir = home_dir.join("Library");
    let agents_dir = library_dir.join("LaunchAgents");

    let plist_file = agents_dir.join(MAC_PLIST_FILENAME);
    if plist_file.exists() {
        let _res = Command::new("launchctl")
            .args(&["unload", "-w", plist_file.to_str().unwrap_or_default()])
            .output();

        std::fs::remove_file(&plist_file).expect("Could not remove espanso entry");

        println!("Entry removed correctly!")
    }else{
        println!("espanso is not installed");
    }
}

#[cfg(target_os = "windows")]
pub fn install(config_set: ConfigSet) {
    println!("Windows does not support system daemon integration.")
}