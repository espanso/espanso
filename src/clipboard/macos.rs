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
use crate::bridge::macos::{get_clipboard, set_clipboard};
use std::ffi::{CStr, CString};

pub struct MacClipboardManager {

}

impl super::ClipboardManager for MacClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        unsafe {
            let mut buffer : [c_char; 2000] = [0; 2000];
            let res = get_clipboard(buffer.as_mut_ptr(), buffer.len() as i32);

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

    fn set_clipboard(&self, payload: &str) {
        let res = CString::new(payload);
        if let Ok(cstr) = res {
            unsafe {
                set_clipboard(cstr.as_ptr());
            }
        }
    }
}

impl MacClipboardManager {
    pub fn new() -> MacClipboardManager {
        MacClipboardManager{}
    }
}