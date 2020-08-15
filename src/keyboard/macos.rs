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
use crate::config::Configs;
use log::error;
use std::ffi::CString;

pub struct MacKeyboardManager {}

impl super::KeyboardManager for MacKeyboardManager {
    fn send_string(&self, _: &Configs, s: &str) {
        let res = CString::new(s);
        match res {
            Ok(cstr) => unsafe {
                send_string(cstr.as_ptr());
            },
            Err(e) => panic!(e.to_string()),
        }
    }

    fn send_enter(&self, _: &Configs) {
        unsafe {
            // Send the kVK_Return key press
            send_vkey(0x24);
        }
    }

    fn trigger_paste(&self, active_config: &Configs) {
        unsafe {
            match active_config.paste_shortcut {
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

    fn trigger_copy(&self, _: &Configs) {
        unsafe {
            trigger_copy();
        }
    }

    fn delete_string(&self, _: &Configs, count: i32) {
        unsafe { delete_string(count) }
    }

    fn move_cursor_left(&self, _: &Configs, count: i32) {
        unsafe {
            // Simulate the Left arrow count times
            send_multi_vkey(0x7B, count);
        }
    }
}

pub fn wait_for_modifiers_release() -> bool {
    let start = std::time::SystemTime::now();
    while start.elapsed().unwrap_or_default().as_millis() < 3000 {
        let pressed = unsafe { crate::bridge::macos::are_modifiers_pressed() };
        if pressed == 0 {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}
