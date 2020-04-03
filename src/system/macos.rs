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
use crate::bridge::macos::{get_active_app_bundle, get_active_app_identifier, get_secure_input_process, get_path_from_pid};

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
    /// Return None if no application has claimed SecureInput, its PID otherwise.
    pub fn get_secure_input_pid() -> Option<i64> {
        unsafe {
            let mut pid: i64 = -1;
            let res = get_secure_input_process(&mut pid as *mut i64);

            if res > 0{
                Some(pid)
            }else{
                None
            }
        }
    }

    /// Check whether an application is currently holding the Secure Input.
    /// Return None if no application has claimed SecureInput, Some((AppName, AppPath)) otherwise.
    pub fn get_secure_input_application() -> Option<(String, String)> {
        use regex::Regex;

        lazy_static! {
            static ref APP_REGEX: Regex = Regex::new("/([^/]+).app/").unwrap();
        };

        unsafe {
            let pid = MacSystemManager::get_secure_input_pid();

            if let Some(pid) = pid {
                // Size of the buffer is ruled by the PROC_PIDPATHINFO_MAXSIZE constant.
                // the underlying proc_pidpath REQUIRES a buffer of that dimension, otherwise it fail silently.
                let mut buffer : [c_char; 4096] = [0; 4096];
                let res = get_path_from_pid(pid, buffer.as_mut_ptr(), buffer.len() as i32);

                if res > 0 {
                    let c_string = CStr::from_ptr(buffer.as_ptr());
                    let string = c_string.to_str();
                    if let Ok(path) = string {
                        if !path.trim().is_empty() {
                            let process = path.trim().to_string();
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
                    }else{
                        None
                    }
                }else{
                    None
                }
            }else{
                None
            }
        }
    }
}