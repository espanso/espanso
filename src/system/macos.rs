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

use std::os::raw::c_char;

use std::ffi::CStr;
use crate::bridge::macos::{get_active_app_bundle, get_active_app_identifier};

pub struct MacSystemManager {

}

impl super::SystemManager for MacSystemManager {
    fn get_current_window_title(&self) -> Option<String> {
        self.get_current_window_class()
    }

    fn get_current_window_class(&self) -> Option<String> {
        unsafe {
            let mut buffer : [c_char; 250] = [0; 250];
            let res = get_active_app_identifier(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = CStr::from_ptr(buffer.as_ptr());

                let string = c_string.to_str();
                if let Ok(string) = string {
                    return Some((*string).to_owned());
                }
            }
        }

        None
    }

    fn get_current_window_executable(&self) -> Option<String> {
        unsafe {
            let mut buffer : [c_char; 250] = [0; 250];
            let res = get_active_app_bundle(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = CStr::from_ptr(buffer.as_ptr());

                let string = c_string.to_str();
                if let Ok(string) = string {
                    return Some((*string).to_owned());
                }
            }
        }

        None
    }
}

impl MacSystemManager {
    pub fn new() -> MacSystemManager {
        MacSystemManager{

        }
    }

    /// Check whether an application is currently holding the Secure Input.
    /// Return None if no application has claimed SecureInput, Some((AppName, AppPath)) otherwise.
    pub fn get_secure_input_application() -> Option<(String, String)> {
        use std::process::Command;
        use regex::Regex;

        let output = Command::new("ioreg")
            .arg("-d")
            .arg("1")
            .arg("-k")
            .arg("IOConsoleUsers")
            .arg("-w")
            .arg("0")
            .output();

        lazy_static! {
            static ref PID_REGEX: Regex = Regex::new("\"kCGSSessionSecureInputPID\"=(\\d+)").unwrap();
        };

        lazy_static! {
            static ref APP_REGEX: Regex = Regex::new("/([^/]+).app/").unwrap();
        };

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(output.stdout.as_slice());
            let caps = PID_REGEX.captures(&output_str);

            if let Some(caps) = caps {
                // Get the PID of the process that is handling SecureInput
                let pid_str = caps.get(1).map_or("", |m| m.as_str());
                let pid = pid_str.parse::<i32>().expect("Invalid pid value");

                // Find the process that is handling the SecureInput
                let output = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .arg("-o")
                    .arg("command=")
                    .output();

                if let Ok(output) = output {
                    let output_str = String::from_utf8_lossy(output.stdout.as_slice());

                    if !output_str.trim().is_empty() {
                        let process = output_str.trim().to_string();
                        let caps = APP_REGEX.captures(&process);
                        let app_name = if let Some(caps) = caps {
                            caps.get(1).map_or("", |m| m.as_str()).to_owned()
                        }else{
                            process.to_owned()
                        };

                        Some((app_name, process))
                    }else{
                        None
                    }
                }else{  // Can't obtain process name
                    None
                }
            }else{ // No process is holding SecureInput
                None
            }
        }else{  // Can't execute the query to the IOKit registry
            None
        }
    }
}