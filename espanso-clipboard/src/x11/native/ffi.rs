/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

use std::os::raw::c_char;

#[link(name = "espansoclipboardx11", kind = "static")]
extern "C" {
  pub fn clipboard_x11_get_text(buffer: *mut c_char, buffer_size: i32) -> i32;
  pub fn clipboard_x11_set_text(text: *const c_char) -> i32;
  pub fn clipboard_x11_set_html(html: *const c_char, fallback_text: *const c_char) -> i32;
  pub fn clipboard_x11_set_image(buffer: *const u8, buffer_size: i32) -> i32;
}
