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

// INSTALLATION

#[cfg(target_os = "macos")]
const MAC_PLIST_CONTENT : &str = include_str!("res/mac/com.federicoterzi.espanso.plist");
#[cfg(target_os = "macos")]
const MAC_PLIST_FILENAME : &str = "com.federicoterzi.espanso.plist";

#[cfg(target_os = "macos")]
pub fn register(_config_set: ConfigSet) {
    use std::fs::create_dir_all;
    use std::process::{Command, ExitStatus};

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
pub fn unregister(_config_set: ConfigSet) {
    use std::fs::create_dir_all;
    use std::process::{Command, ExitStatus};

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

// LINUX

#[cfg(target_os = "linux")]
const LINUX_SERVICE_CONTENT : &str = include_str!("res/linux/systemd.service");
#[cfg(target_os = "linux")]
const LINUX_SERVICE_FILENAME : &str = "espanso.service";

#[cfg(target_os = "linux")]
pub fn register(config_set: ConfigSet) {
    use std::fs::create_dir_all;
    use std::process::{Command, ExitStatus};

    // Check if espanso service is already registered
    let res = Command::new("systemctl")
        .args(&["--user", "is-enabled", "espanso"])
        .output();
    if let Ok(res) = res {
        let output = String::from_utf8_lossy(res.stdout.as_slice());
        let output = output.trim();
        if res.status.success() && output == "enabled" {
            eprintln!("espanso service is already registered to systemd");
            eprintln!("If you want to register it again, please uninstall it first with:");
            eprintln!("    espanso unregister");
            std::process::exit(5);
        }
    }

    // User level systemd services should be placed in this directory:
    // $XDG_CONFIG_HOME/systemd/user/, usually: ~/.config/systemd/user/
    let config_dir = dirs::config_dir().expect("Could not get configuration directory");
    let systemd_dir = config_dir.join("systemd");
    let user_dir = systemd_dir.join("user");

    // Make sure the directory exists
    if !user_dir.exists() {
        create_dir_all(user_dir.clone()).expect("Could not create systemd user directory");
    }

    let service_file = user_dir.join(LINUX_SERVICE_FILENAME);
    if !service_file.exists() {
        println!("Creating service entry: {}", service_file.to_str().unwrap_or_default());

        let espanso_path = std::env::current_exe().expect("Could not get espanso executable path");
        println!("Entry will point to: {}", espanso_path.to_str().unwrap_or_default());

        let service_content = String::from(LINUX_SERVICE_CONTENT)
            .replace("{{{espanso_path}}}", espanso_path.to_str().unwrap_or_default());

        std::fs::write(service_file.clone(), service_content).expect("Unable to write service file");

        println!("Service file created correctly!")
    }

    println!("Enabling espanso for systemd...");

    let res = Command::new("systemctl")
        .args(&["--user", "enable", "espanso"])
        .status();

    if let Ok(status) = res {
        if status.success() {
            println!("Service registered correctly!")
        }
    }else{
        println!("Error loading espanso service");
    }
}

#[cfg(target_os = "linux")]
pub fn unregister(config_set: ConfigSet) {
    use std::process::{Command, ExitStatus};

    // Disable the service first
    let res = Command::new("systemctl")
        .args(&["--user", "disable", "espanso"])
        .status();

    // Then delete the espanso.service entry
    let config_dir = dirs::config_dir().expect("Could not get configuration directory");
    let systemd_dir = config_dir.join("systemd");
    let user_dir = systemd_dir.join("user");
    let service_file = user_dir.join(LINUX_SERVICE_FILENAME);

    if service_file.exists() {
        let res = std::fs::remove_file(&service_file);
        match res {
            Ok(_) => {
                println!("Deleted entry at {}", service_file.to_string_lossy());
                println!("Service unregistered successfully!");
            },
            Err(e) => {
                println!("Error, could not delete service entry at {} with error {}",
                         service_file.to_string_lossy(), e);
            },
        }
    }else{
        eprintln!("Error, could not find espanso service file");
    }
}

// WINDOWS

#[cfg(target_os = "windows")]
pub fn register(_config_set: ConfigSet) {
    println!("Windows does not support automatic system daemon integration.")
}

#[cfg(target_os = "windows")]
pub fn unregister(_config_set: ConfigSet) {
    println!("Windows does not support automatic system daemon integration.")
}