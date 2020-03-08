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

use std::ffi::CString;
use crate::bridge::linux::*;
use super::PasteShortcut;
use log::error;

pub struct LinuxKeyboardManager {
}

impl super::KeyboardManager for LinuxKeyboardManager {
    fn send_string(&self, s: &str) {
        let res = CString::new(s);
        match res {
            Ok(cstr) => unsafe { send_string(cstr.as_ptr()); }
            Err(e) => panic!(e.to_string())
        }
    }

    fn send_enter(&self) {
        unsafe {
            send_enter();
        }
    }

    fn trigger_paste(&self, shortcut: &PasteShortcut) {
        unsafe {
            match shortcut {
                PasteShortcut::Default => {
                    let is_special = is_current_window_special();

                    // Terminals use a different keyboard combination to paste from clipboard,
                    // so we need to check the correct situation.
                    if is_special == 0 {
                        trigger_paste();
                    }else if is_special == 2 {  // Special case for stterm
                        trigger_alt_shift_ins_paste();
                    }else if is_special == 3 {  // Special case for Emacs
                        trigger_shift_ins_paste();
                    }else if is_special == 4 {  // CTRL+ALT+V used in some terminals (urxvt)
                        trigger_ctrl_alt_paste();
                    }else{
                        trigger_terminal_paste();
                    }
                },
                PasteShortcut::CtrlV => {
                    trigger_paste();
                },
                PasteShortcut::CtrlShiftV => {
                    trigger_terminal_paste();
                },
                PasteShortcut::ShiftInsert=> {
                    trigger_shift_ins_paste();
                },
                PasteShortcut::CtrlAltV => {
                    trigger_ctrl_alt_paste();
                },
                _ => {
                    error!("Linux backend does not support this Paste Shortcut, please open an issue on GitHub if you need it.")
                }
            }
        }
    }

    fn delete_string(&self, count: i32) {
        unsafe {delete_string(count)}
    }

    fn move_cursor_left(&self, count: i32) {
        unsafe {
            left_arrow(count);
        }
    }

    fn trigger_copy(&self) {
        unsafe {
            trigger_copy();
        }
    }
}