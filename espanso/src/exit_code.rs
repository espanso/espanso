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

pub const WORKER_SUCCESS: i32 = 0;
pub const WORKER_ALREADY_RUNNING: i32 = 1;
pub const WORKER_GENERAL_ERROR: i32 = 2;
pub const WORKER_LEGACY_ALREADY_RUNNING: i32 = 3;
pub const WORKER_EXIT_ALL_PROCESSES: i32 = 101;
pub const WORKER_RESTART: i32 = 102;

pub const DAEMON_SUCCESS: i32 = 0;
pub const DAEMON_ALREADY_RUNNING: i32 = 1;
pub const DAEMON_GENERAL_ERROR: i32 = 2;
pub const DAEMON_LEGACY_ALREADY_RUNNING: i32 = 3;

pub const MIGRATE_SUCCESS: i32 = 0;
pub const MIGRATE_ALREADY_NEW_FORMAT: i32 = 1;
pub const MIGRATE_LEGACY_INSTANCE_RUNNING: i32 = 2;
pub const MIGRATE_USER_ABORTED: i32 = 3;
pub const MIGRATE_CLEAN_FAILURE: i32 = 50;
pub const MIGRATE_DIRTY_FAILURE: i32 = 51;
pub const MIGRATE_UNEXPECTED_FAILURE: i32 = 101;

pub const ADD_TO_PATH_SUCCESS: i32 = 0;
pub const ADD_TO_PATH_FAILURE: i32 = 1;

use std::sync::Mutex;

lazy_static! {
  static ref CURRENT_PANIC_EXIT_CODE: Mutex<i32> = Mutex::new(MIGRATE_UNEXPECTED_FAILURE);
}

pub fn configure_custom_panic_hook() {
  let previous_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    (*previous_hook)(info);

    // Part of this code is taken from the "rust-log-panics" crate
    let thread = std::thread::current();
    let thread = thread.name().unwrap_or("<unnamed>");

    let msg = match info.payload().downcast_ref::<&'static str>() {
      Some(s) => *s,
      None => match info.payload().downcast_ref::<String>() {
        Some(s) => &**s,
        None => "Box<Any>",
      },
    };

    match info.location() {
      Some(location) => {
        eprintln!(
          "ERROR: '{}' panicked at '{}': {}:{}",
          thread,
          msg,
          location.file(),
          location.line(),
        );
      }
      None => eprintln!("ERROR: '{}' panicked at '{}'", thread, msg,),
    }

    let exit_code = CURRENT_PANIC_EXIT_CODE.lock().unwrap();
    std::process::exit(*exit_code);
  }));
}

pub fn update_panic_exit_code(exit_code: i32) {
  let mut lock = CURRENT_PANIC_EXIT_CODE
    .lock()
    .expect("unable to update panic exit code");
  *lock = exit_code;
}

