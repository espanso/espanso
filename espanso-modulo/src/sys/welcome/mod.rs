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

use std::os::raw::c_int;
use std::sync::Mutex;

use crate::sys::util::convert_to_cstring_or_null;
use crate::{
  sys::interop::WelcomeMetadata,
  welcome::{WelcomeHandlers, WelcomeOptions},
};
use lazy_static::lazy_static;

lazy_static! {
  static ref HANDLERS: Mutex<Option<WelcomeHandlers>> = Mutex::new(None);
}

pub fn show(options: WelcomeOptions) {
  let (_c_window_icon_path, c_window_icon_path_ptr) =
    convert_to_cstring_or_null(options.window_icon_path);
  let (_c_tray_image_path, c_tray_image_path_ptr) =
    convert_to_cstring_or_null(options.tray_image_path);

  extern "C" fn dont_show_again_changed(dont_show: c_int) {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in dont_show_again_changed method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.dont_show_again_changed.as_ref() {
      let value = dont_show == 1;
      (*handler_ref)(value);
    }
  }

  {
    let mut lock = HANDLERS.lock().expect("unable to acquire handlers lock");
    *lock = Some(options.handlers);
  }

  let welcome_metadata = WelcomeMetadata {
    window_icon_path: c_window_icon_path_ptr,
    tray_image_path: c_tray_image_path_ptr,
    already_running: i32::from(options.is_already_running),
    dont_show_again_changed,
  };

  unsafe {
    super::interop::interop_show_welcome(&welcome_metadata);
  }
}
