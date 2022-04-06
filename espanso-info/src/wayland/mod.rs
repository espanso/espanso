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

// This file:
// Last changes: 2022-04-05T17:55:36+02:00 by Hendrik G. Seliger (github@hseliger.eu)

use std::{ffi::CStr, os::raw::c_char};

use crate::{AppInfo, AppInfoProvider};

use self::ffi::{info_get_class, info_get_exec, info_get_title};

mod ffi;

pub(crate) struct WaylandAppInfoProvider {}

impl WaylandAppInfoProvider {
 pub fn new() -> Self {
   Self {}
 }
}

impl AppInfoProvider for WaylandAppInfoProvider {
 fn get_info(&self) -> AppInfo {
   AppInfo {
     title: self.get_title(),
     class: self.get_class(),
     exec: self.get_exec(),
   }
 }
}

impl WaylandAppInfoProvider {
 fn get_exec(&self) -> Option<String> {
   let mut buffer: [c_char; 2048] = [0; 2048];
   if unsafe { info_get_exec(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) } > 0 {
     let string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
     let string = string.to_string_lossy();
     if !string.is_empty() {
       Some(string.to_string())
     } else {
       None
     }
   } else {
     None
   }
 }

 fn get_class(&self) -> Option<String> {
   let mut buffer: [c_char; 2048] = [0; 2048];
   if unsafe { info_get_class(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) } > 0 {
     let string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
     let string = string.to_string_lossy();
     if !string.is_empty() {
       Some(string.to_string())
     } else {
       None
     }
   } else {
     None
   }
 }

 fn get_title(&self) -> Option<String> {
   let mut buffer: [c_char; 2048] = [0; 2048];
   if unsafe { info_get_title(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) } > 0 {
     let string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
     let string = string.to_string_lossy();
     if !string.is_empty() {
       Some(string.to_string())
     } else {
       None
     }
   } else {
     None
   }
 }
}
