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

use std::process::Command;
use super::MenuItem;
use log::{error, info};
use std::path::PathBuf;

const LINUX_ICON_CONTENT : &[u8] = include_bytes!("../res/linux/icon.png");

pub struct LinuxUIManager {
    icon_path: PathBuf,
}

impl super::UIManager for LinuxUIManager {
    fn notify(&self, message: &str) {
        let res = Command::new("notify-send")
                        .args(&["-i", self.icon_path.to_str().unwrap_or_default(),
                            "-t", "2000", "espanso", message])
                        .output();

        if let Err(e) = res {
            error!("Could not send a notification, error: {}", e);
        }
    }

    fn show_menu(&self, _menu: Vec<MenuItem>) {
        // Not implemented on linux
    }

    fn cleanup(&self) {
        // Nothing to do here
    }
}

impl LinuxUIManager {
    pub fn new() -> LinuxUIManager {
        // Initialize the icon if not present
        let data_dir = crate::context::get_data_dir();
        let icon_path = data_dir.join("icon.png");
        if !icon_path.exists() {
            info!("Creating espanso icon in '{}'", icon_path.to_str().unwrap_or_default());
            std::fs::write(&icon_path, LINUX_ICON_CONTENT).expect("Unable to copy espanso icon");
        }

        LinuxUIManager{
            icon_path
        }
    }
}