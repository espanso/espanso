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

use std::ffi::CString;
use std::os::raw::c_char;

pub fn convert_to_cstring_or_null(str: Option<String>) -> (Option<CString>, *const c_char) {
  let c_string =
    str.map(|str| CString::new(str).expect("unable to convert Option<String> to CString"));
  let c_ptr = c_string
    .as_ref()
    .map_or(std::ptr::null(), |path| path.as_ptr());

  (c_string, c_ptr)
}
