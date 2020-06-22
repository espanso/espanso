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

use std::os::raw::{c_char, c_void};

#[allow(improper_ctypes)]
#[link(name = "linuxbridge", kind = "static")]
extern "C" {
    pub fn check_x11() -> i32;
    pub fn initialize(s: *const c_void) -> i32;
    pub fn eventloop();
    pub fn cleanup();

    // System
    pub fn get_active_window_name(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_window_class(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_window_executable(buffer: *mut c_char, size: i32) -> i32;
    pub fn is_current_window_special() -> i32;
    pub fn register_error_callback(
        cb: extern "C" fn(_self: *mut c_void, error_code: c_char, request_code: c_char, minor_code: c_char),
    );

    // Keyboard
    pub fn register_keypress_callback(
        cb: extern "C" fn(_self: *mut c_void, *const u8, i32, i32, i32),
    );

    pub fn send_string(string: *const c_char);
    pub fn delete_string(count: i32);
    pub fn left_arrow(count: i32);
    pub fn send_enter();
    pub fn trigger_paste();
    pub fn trigger_terminal_paste();
    pub fn trigger_shift_ins_paste();
    pub fn trigger_alt_shift_ins_paste();
    pub fn trigger_ctrl_alt_paste();
    pub fn trigger_copy();

    pub fn fast_send_string(string: *const c_char, delay: i32);
    pub fn fast_delete_string(count: i32, delay: i32);
    pub fn fast_left_arrow(count: i32);
    pub fn fast_send_enter();
}
