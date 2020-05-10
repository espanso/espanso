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

use crate::bridge::windows::{get_clipboard, set_clipboard, set_clipboard_image};
use std::path::Path;
use widestring::U16CString;

pub struct WindowsClipboardManager {}

impl WindowsClipboardManager {
    pub fn new() -> WindowsClipboardManager {
        WindowsClipboardManager {}
    }
}

impl super::ClipboardManager for WindowsClipboardManager {
    fn get_clipboard(&self) -> Option<String> {
        unsafe {
            let mut buffer: [u16; 2000] = [0; 2000];
            let res = get_clipboard(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }

    fn set_clipboard(&self, payload: &str) {
        unsafe {
            let payload_c = U16CString::from_str(payload).unwrap();
            set_clipboard(payload_c.as_ptr());
        }
    }

    fn set_clipboard_image(&self, image_path: &Path) {
        let path_string = image_path.to_string_lossy().into_owned();
        unsafe {
            let payload_c = U16CString::from_str(path_string).unwrap();
            set_clipboard_image(payload_c.as_ptr());
        }
    }
}
