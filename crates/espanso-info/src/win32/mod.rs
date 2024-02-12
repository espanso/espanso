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

use widestring::U16CStr;

use crate::{AppInfo, AppInfoProvider};

use self::ffi::{info_get_exec, info_get_title};

mod ffi;

pub struct WinAppInfoProvider {}

impl WinAppInfoProvider {
  pub fn new() -> Self {
    Self {}
  }
}

impl AppInfoProvider for WinAppInfoProvider {
  fn get_info(&self) -> AppInfo {
    AppInfo {
      title: WinAppInfoProvider::get_title(),
      class: None,
      exec: WinAppInfoProvider::get_exec(),
    }
  }
}

impl WinAppInfoProvider {
  fn get_exec() -> Option<String> {
    let mut buffer: [u16; 2048] = [0; 2048];
    if unsafe { info_get_exec(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) } == 0 {
      None
    } else {
      let string = unsafe { U16CStr::from_ptr_str(buffer.as_ptr()) };
      let string = string.to_string_lossy();
      if string.is_empty() {
        None
      } else {
        Some(string)
      }
    }
  }

  fn get_title() -> Option<String> {
    let mut buffer: [u16; 2048] = [0; 2048];
    if unsafe { info_get_title(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) } > 0 {
      let string = unsafe { U16CStr::from_ptr_str(buffer.as_ptr()) };
      let string = string.to_string_lossy();
      if string.is_empty() {
        None
      } else {
        Some(string)
      }
    } else {
      None
    }
  }
}
