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

#[cfg(target_os = "macos")]
use lazy_static::lazy_static;
#[cfg(target_os = "macos")]
use std::{ffi::CStr, os::raw::c_char};

mod ffi;

/// Check whether an application is currently holding the Secure Input.
/// Return None if no application has claimed `SecureInput`, its PID otherwise.
#[cfg(target_os = "macos")]
pub fn get_secure_input_pid() -> Option<i64> {
  unsafe {
    let mut pid: i64 = -1;
    let res = ffi::mac_utils_get_secure_input_process(std::ptr::from_mut::<i64>(&mut pid));

    if res > 0 {
      Some(pid)
    } else {
      None
    }
  }
}

/// Check whether an application is currently holding the Secure Input.
/// Return None if no application has claimed `SecureInput`, `Some((AppName, AppPath))` otherwise.
#[cfg(target_os = "macos")]
pub fn get_secure_input_application() -> Option<(String, String)> {
  unsafe {
    let pid = get_secure_input_pid();

    if let Some(pid) = pid {
      // Size of the buffer is ruled by the PROC_PIDPATHINFO_MAXSIZE constant.
      // the underlying proc_pidpath REQUIRES a buffer of that dimension, otherwise it fail silently.
      let mut buffer: [c_char; 4096] = [0; 4096];
      let res = ffi::mac_utils_get_path_from_pid(pid, buffer.as_mut_ptr(), buffer.len() as i32);

      if res > 0 {
        let c_string = CStr::from_ptr(buffer.as_ptr());
        let string = c_string.to_str();
        if let Ok(path) = string {
          if !path.trim().is_empty() {
            let process = path.trim().to_string();
            let app_name = if let Some(name) = get_app_name_from_path(&process) {
              name
            } else {
              process.clone()
            };

            return Some((app_name, process));
          }
        }
      }
    }

    None
  }
}

#[cfg(target_os = "macos")]
fn get_app_name_from_path(path: &str) -> Option<String> {
  use regex::Regex;

  lazy_static! {
    static ref APP_REGEX: Regex = Regex::new("/([^/]+).(app|bundle)/").unwrap();
  };

  let caps = APP_REGEX.captures(path);
  caps.map(|caps| caps.get(1).map_or("", |m| m.as_str()).to_owned())
}

#[cfg(target_os = "macos")]
pub fn check_accessibility() -> bool {
  unsafe { ffi::mac_utils_check_accessibility() > 0 }
}

#[cfg(target_os = "macos")]
pub fn prompt_accessibility() -> bool {
  unsafe { ffi::mac_utils_prompt_accessibility() > 0 }
}

#[cfg(target_os = "macos")]
pub fn convert_to_foreground_app() {
  unsafe {
    ffi::mac_utils_transition_to_foreground_app();
  }
}

#[cfg(target_os = "macos")]
pub fn convert_to_background_app() {
  unsafe {
    ffi::mac_utils_transition_to_background_app();
  }
}

#[cfg(target_os = "macos")]
pub fn start_headless_eventloop() {
  unsafe {
    ffi::mac_utils_start_headless_eventloop();
  }
}

#[cfg(target_os = "macos")]
pub fn exit_headless_eventloop() {
  unsafe {
    ffi::mac_utils_exit_headless_eventloop();
  }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
  use super::*;

  #[test]
  fn test_get_app_name_from_path() {
    let app_name = get_app_name_from_path("/Applications/iTerm.app/Contents/MacOS/iTerm2");
    assert_eq!(app_name.unwrap(), "iTerm");
  }

  #[test]
  fn test_get_app_name_from_path_no_app_name() {
    let app_name = get_app_name_from_path("/another/directory");
    assert!(app_name.is_none());
  }

  #[test]
  fn test_get_app_name_from_path_security_bundle() {
    let app_name = get_app_name_from_path("/System/Library/Frameworks/Security.framework/Versions/A/MachServices/SecurityAgent.bundle/Contents/MacOS/SecurityAgent");
    assert_eq!(app_name.unwrap(), "SecurityAgent");
  }
}
