/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

use crate::{AppInfo, AppInfoProvider};

use std::process::Command;

pub(crate) struct WaylandAppInfoProvider {}

fn empty_app_info() -> AppInfo {
    AppInfo {
        title: None,
        exec: Some("Missing kdotool or non-KDE environment!".into()),
        class: None,
    }
}

impl WaylandAppInfoProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppInfoProvider for WaylandAppInfoProvider {
    // TODO: can we read these info on Wayland?
    // maybe
    fn get_info(&self) -> AppInfo {
        let class = match Command::new("kdotool")
            .arg("getactivewindow")
            .arg("getwindowclassname")
            .output()
        {
            Ok(out) => {
                let class_ = String::from_utf8(out.stdout).expect("Error decoding from utf8");
                Some(class_)
            }
            Err(_) => return empty_app_info(),
        };

        let title = match Command::new("kdotool")
            .arg("getactivewindow")
            .arg("getwindowname")
            .output()
        {
            Ok(out) => {
                let title_ = String::from_utf8(out.stdout).expect("Error decoding from utf8");
                Some(title_)
            }
            Err(_) => None,
        };

        let exec = match Command::new("kdotool")
            .arg("getactivewindow")
            .arg("getwindowpid")
            .output()
        {
            Ok(out) => {
                let pid_ = String::from_utf8(out.stdout).expect("Error decoding from utf8");
                match Command::new("readlink")
                    .arg(format!("/proc/{}", pid_))
                    .output()
                {
                    Ok(out) => {
                        let exec_ =
                            String::from_utf8(out.stdout).expect("Error decoding from utf8");
                        Some(exec_)
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        AppInfo { title, exec, class }
    }
}
