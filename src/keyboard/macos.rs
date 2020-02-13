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

use super::PasteShortcut;
use crate::bridge::macos::*;
use log::error;
use std::ffi::CString;

pub struct MacKeyboardManager {}

impl super::KeyboardManager for MacKeyboardManager {
    fn send_string(&self, s: &str) {
        let res = CString::new(s);
        match res {
            Ok(cstr) => unsafe {
                send_string(cstr.as_ptr());
            },
            Err(e) => panic!(e.to_string()),
        }
    }

    fn send_enter(&self) {
        unsafe {
            // Send the kVK_Return key press
            send_vkey(0x24);
        }
    }

    fn trigger_paste(&self, shortcut: &PasteShortcut) {
        unsafe {
            match shortcut {
                PasteShortcut::Default => {
                    unsafe {
                        trigger_paste();
                    }
                },
                _ => {
                    error!("MacOS backend does not support this Paste Shortcut, please open an issue on GitHub if you need it.")
                }
            }
        }
    }

    fn trigger_copy(&self) {
        unsafe {
            trigger_copy();
        }
    }

    fn delete_string(&self, count: i32) {
        unsafe { delete_string(count) }
    }

    fn move_cursor_left(&self, count: i32) {
        unsafe {
            // Simulate the Left arrow count times
            send_multi_vkey(0x7B, count);
        }
    }
}
