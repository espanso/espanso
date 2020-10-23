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

use crate::bridge::macos::*;
use log::{error, warn};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

pub struct MacClipboardManager {}

impl super::ClipboardManager for MacClipboardManager {
    fn get_clipboard(&self) -> Option<String> {
        unsafe {
            let mut buffer: [c_char; 2000] = [0; 2000];
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

    fn set_clipboard_image(&self, image_path: &Path) {
        let path_string = image_path.to_string_lossy().into_owned();
        let res = CString::new(path_string);
        if let Ok(path) = res {
            unsafe {
                let result = set_clipboard_image(path.as_ptr());
                if result != 1 {
                    warn!("Couldn't set clipboard for image: {:?}", image_path)
                }
            }
        }
    }

    fn set_clipboard_html(&self, html: &str) {
        // Render the text fallback for those applications that don't support HTML clipboard
        let decorator = html2text::render::text_renderer::TrivialDecorator::new();
        let text_fallback =
            html2text::from_read_with_decorator(html.as_bytes(), 1000000, decorator);
        unsafe {
            let payload_c =
                CString::new(html).expect("unable to create CString for html content");
            let payload_fallback_c = CString::new(text_fallback).unwrap();
            set_clipboard_html(payload_c.as_ptr(), payload_fallback_c.as_ptr());
        }
    }
}

impl MacClipboardManager {
    pub fn new() -> MacClipboardManager {
        MacClipboardManager {}
    }
}
