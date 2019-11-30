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

use widestring::{U16CString};
use crate::bridge::windows::*;
use super::PasteShortcut;
use log::error;

pub struct WindowsKeyboardManager {
}

impl super::KeyboardManager for WindowsKeyboardManager {
    fn send_string(&self, s: &str) {
        let res = U16CString::from_str(s);
        match res {
            Ok(s) => {
                unsafe {
                    send_string(s.as_ptr());
                }
            }
            Err(e) => println!("Error while sending string: {}", e.to_string())
        }

    }

    fn send_enter(&self) {
        unsafe {
            // Send the VK_RETURN key press
            send_vkey(0x0D);
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
                    error!("Windows backend does not support this Paste Shortcut, please open an issue on GitHub if you need it.")
                }
            }
        }
    }

    fn delete_string(&self, count: i32) {
        unsafe {
            delete_string(count)
        }
    }

    fn move_cursor_left(&self, count: i32) {
        unsafe {
            // Send the left arrow key multiple times
            send_multi_vkey(0x25, count)
        }
    }
}