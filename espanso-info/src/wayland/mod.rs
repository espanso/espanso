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

pub(crate) struct WaylandEmptyAppInfoProvider {}
pub(crate) struct WaylandKDEAppInfoProvider {}
pub(crate) struct WaylandNiriAppInfoProvider {}

fn empty_app_info() -> AppInfo {
    AppInfo {
        title: None,
        exec: None,
        class: None,
    }
}

// for unsupported DEs/WMs
impl WaylandEmptyAppInfoProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppInfoProvider for WaylandEmptyAppInfoProvider {
    fn get_info(&self) -> AppInfo {
        empty_app_info()
    }
}

// for KDE with kdotool
impl WaylandKDEAppInfoProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppInfoProvider for WaylandKDEAppInfoProvider {
    fn get_info(&self) -> AppInfo {
        let class = if let Ok(out) = Command::new("kdotool")
            .arg("getactivewindow")
            .arg("getwindowclassname")
            .output()
        {
            let mut __stdout = out.stdout;
            if !__stdout.is_empty() {
                __stdout.pop();
            }
            let class_ = String::from_utf8(__stdout).expect("Error decoding from utf8");
            Some(class_)
        } else {
            // kdotool is checked once in the startup of main.rs
            // we do not need to log it here again
            return empty_app_info();
        };

        let title = match Command::new("kdotool")
            .arg("getactivewindow")
            .arg("getwindowname")
            .output()
        {
            Ok(out) => {
                let mut __stdout = out.stdout;
                if !__stdout.is_empty() {
                    __stdout.pop();
                }
                let title_ = String::from_utf8(__stdout).expect("Error decoding from utf8");
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
                let mut __stdout = out.stdout;
                if !__stdout.is_empty() {
                    __stdout.pop();
                }
                let pid_ = String::from_utf8(__stdout).expect("Error decoding from utf8");
                match Command::new("readlink")
                    .arg(format!("/proc/{pid_}/exe"))
                    .output()
                {
                    Ok(out) => {
                        let mut __stdout = out.stdout;
                        if !__stdout.is_empty() {
                            __stdout.pop();
                        }
                        let exec_ = String::from_utf8(__stdout).expect("Error decoding from utf8");
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

// for Niri
impl WaylandNiriAppInfoProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppInfoProvider for WaylandNiriAppInfoProvider {
    fn get_info(&self) -> AppInfo {
        if let Ok(out) = Command::new("niri")
            .arg("msg")
            .arg("focused-window")
            .output()
        {
            let txt = String::from_utf8(out.stdout).expect("Error decoding from utf8");

            let mut title = None;
            let mut class = None;
            let mut exec = None;

            for line in txt.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("Title:") {
                    title = Some(rest.trim().trim_matches('"').to_string());
                } else if let Some(rest) = trimmed.strip_prefix("App ID:") {
                    class = Some(rest.trim().trim_matches('"').to_string());
                } else if let Some(rest) = trimmed.strip_prefix("PID:") {
                    let pid_ = rest.trim().to_string();
                    exec = match Command::new("readlink")
                        .arg(format!("/proc/{pid_}/exe"))
                        .output()
                    {
                        Ok(out) => {
                            let mut __stdout = out.stdout;
                            if !__stdout.is_empty() {
                                __stdout.pop();
                            }
                            let exec_ =
                                String::from_utf8(__stdout).expect("Error decoding from utf8");
                            Some(exec_)
                        }
                        Err(_) => None,
                    }
                }
            }

            return AppInfo { title, exec, class };
        }
        empty_app_info()
    }
}
