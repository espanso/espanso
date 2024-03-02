/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::ffi::CString;

use crate::sys::util::convert_to_cstring_or_null;
use crate::{sys::interop::TextViewMetadata, textview::TextViewOptions};

pub fn show(options: TextViewOptions) {
  let (_c_window_icon_path, c_window_icon_path_ptr) =
    convert_to_cstring_or_null(options.window_icon_path);
  let c_title = CString::new(options.title).expect("unable to convert title to CString");
  let c_content = CString::new(options.content).expect("unable to convert content to CString");

  let textview_metadata = TextViewMetadata {
    window_icon_path: c_window_icon_path_ptr,
    title: c_title.as_ptr(),
    content: c_content.as_ptr(),
  };

  unsafe {
    super::interop::interop_show_text_view(&textview_metadata);
  }
}
