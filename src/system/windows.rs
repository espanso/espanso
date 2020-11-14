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

use crate::bridge::windows::*;
use widestring::U16CString;

pub struct WindowsSystemManager {}

impl WindowsSystemManager {
    pub fn new() -> WindowsSystemManager {
        WindowsSystemManager {}
    }
}

impl super::SystemManager for WindowsSystemManager {
    fn get_current_window_title(&self) -> Option<String> {
        unsafe {
            let mut buffer: [u16; 256] = [0; 256];
            let res = get_active_window_name(buffer.as_mut_ptr(), (buffer.len() - 1) as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }

    fn get_current_window_class(&self) -> Option<String> {
        self.get_current_window_executable()
    }

    fn get_current_window_executable(&self) -> Option<String> {
        unsafe {
            let mut buffer: [u16; 256] = [0; 256];
            let res = get_active_window_executable(buffer.as_mut_ptr(), (buffer.len() - 1) as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }
}
