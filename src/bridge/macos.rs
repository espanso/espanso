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

#[repr(C)]
pub struct MacMenuItem {
    pub item_id: i32,
    pub item_type: i32,
    pub item_name: [c_char; 100],
}

#[allow(improper_ctypes)]
#[link(name = "macbridge", kind = "static")]
extern "C" {
    pub fn initialize(
        s: *const c_void,
        icon_path: *const c_char,
        disabled_icon_path: *const c_char,
        show_icon: i32,
    );
    pub fn eventloop();
    pub fn headless_eventloop();

    // System
    pub fn check_accessibility() -> i32;
    pub fn prompt_accessibility() -> i32;
    pub fn open_settings_panel();
    pub fn get_active_app_bundle(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_app_identifier(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_secure_input_process(pid: *mut i64) -> i32;
    pub fn get_path_from_pid(pid: i64, buffer: *mut c_char, size: i32) -> i32;

    // Clipboard
    pub fn get_clipboard(buffer: *mut c_char, size: i32) -> i32;
    pub fn set_clipboard(text: *const c_char) -> i32;
    pub fn set_clipboard_image(path: *const c_char) -> i32;
    pub fn set_clipboard_html(html: *const c_char, text_fallback: *const c_char) -> i32;

    // UI
    pub fn register_icon_click_callback(cb: extern "C" fn(_self: *mut c_void));
    pub fn show_context_menu(items: *const MacMenuItem, count: i32) -> i32;
    pub fn register_context_menu_click_callback(cb: extern "C" fn(_self: *mut c_void, id: i32));
    pub fn update_tray_icon(enabled: i32);

    // Keyboard
    pub fn register_keypress_callback(
        cb: extern "C" fn(_self: *mut c_void, *const u8, i32, i32, i32),
    );

    pub fn send_string(string: *const c_char);
    pub fn send_vkey(vk: i32);
    pub fn send_multi_vkey(vk: i32, count: i32);
    pub fn delete_string(count: i32);
    pub fn trigger_paste();
    pub fn trigger_copy();
    pub fn are_modifiers_pressed() -> i32;
}
