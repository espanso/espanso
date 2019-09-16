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

use std::os::raw::{c_void};

#[repr(C)]
pub struct WindowsMenuItem {
    pub item_id: i32,
    pub item_type: i32,
    pub item_name: [u16; 100],
}

#[allow(improper_ctypes)]
#[link(name="winbridge", kind="static")]
extern {
    pub fn start_daemon_process() -> i32;
    pub fn initialize(s: *const c_void, ico_path: *const u16, bmp_path: *const u16) -> i32;

    // SYSTEM
    pub fn get_active_window_name(buffer: *mut u16, size: i32) -> i32;
    pub fn get_active_window_executable(buffer: *mut u16, size: i32) -> i32;

    // UI
    pub fn show_notification(message: *const u16) -> i32;
    pub fn close_notification();
    pub fn show_context_menu(items: *const WindowsMenuItem, count: i32) -> i32;
    pub fn register_icon_click_callback(cb: extern fn(_self: *mut c_void));
    pub fn register_context_menu_click_callback(cb: extern fn(_self: *mut c_void, id: i32));

    // CLIPBOARD
    pub fn get_clipboard(buffer: *mut u16, size: i32) -> i32;
    pub fn set_clipboard(payload: *const u16) -> i32;

    // KEYBOARD
    pub fn register_keypress_callback(cb: extern fn(_self: *mut c_void, *const i32,
                                                i32, i32, i32));

    pub fn eventloop();
    pub fn send_string(string: *const u16);
    pub fn send_vkey(vk: i32);
    pub fn delete_string(count: i32);
    pub fn trigger_paste();
}