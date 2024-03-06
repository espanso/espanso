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
pub const WORKER_EXIT_ALL_PROCESSES: i32 = 50;
pub const WORKER_RESTART: i32 = 51;
pub const WORKER_ERROR_EXIT_NO_CODE: i32 = 90;

pub const DAEMON_SUCCESS: i32 = 0;
pub const DAEMON_ALREADY_RUNNING: i32 = 1;
pub const DAEMON_GENERAL_ERROR: i32 = 2;
pub const DAEMON_LEGACY_ALREADY_RUNNING: i32 = 3;
pub const DAEMON_FATAL_CONFIG_ERROR: i32 = 4;

pub const MIGRATE_SUCCESS: i32 = 0;
pub const MIGRATE_ALREADY_NEW_FORMAT: i32 = 1;
pub const MIGRATE_LEGACY_INSTANCE_RUNNING: i32 = 2;
pub const MIGRATE_USER_ABORTED: i32 = 3;
pub const MIGRATE_CLEAN_FAILURE: i32 = 50;
pub const MIGRATE_DIRTY_FAILURE: i32 = 51;
pub const MIGRATE_UNEXPECTED_FAILURE: i32 = 52;

pub const ADD_TO_PATH_SUCCESS: i32 = 0;
pub const ADD_TO_PATH_FAILURE: i32 = 1;

pub const LAUNCHER_SUCCESS: i32 = 0;
pub const LAUNCHER_CONFIG_DIR_POPULATION_FAILURE: i32 = 1;
pub const LAUNCHER_ALREADY_RUNNING: i32 = 2;

pub const SERVICE_SUCCESS: i32 = 0;
pub const SERVICE_FAILURE: i32 = 1;
pub const SERVICE_NOT_REGISTERED: i32 = 2;
pub const SERVICE_ALREADY_RUNNING: i32 = 3;
pub const SERVICE_NOT_RUNNING: i32 = 4;
pub const SERVICE_TIMED_OUT: i32 = 5;

pub const WORKAROUND_SUCCESS: i32 = 0;
#[allow(dead_code)]
pub const WORKAROUND_FAILURE: i32 = 1;
#[allow(dead_code)]
pub const WORKAROUND_NOT_AVAILABLE: i32 = 2;

#[allow(dead_code)]
pub const PACKAGE_SUCCESS: i32 = 0;
#[allow(dead_code)]
pub const PACKAGE_UNEXPECTED_FAILURE: i32 = 1;
pub const PACKAGE_INSTALL_FAILED: i32 = 2;
pub const PACKAGE_UNINSTALL_FAILED: i32 = 3;
pub const PACKAGE_LIST_FAILED: i32 = 4;
pub const PACKAGE_UPDATE_FAILED: i32 = 5;
pub const PACKAGE_UPDATE_PARTIAL_FAILURE: i32 = 6;

#[allow(dead_code)]
pub const UNEXPECTED_RUN_AS_ROOT: i32 = 42;

use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::error_eprintln;

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

    if let Some(location) = info.location() {
      error_eprintln!(
        "ERROR: '{}' panicked at '{}': {}:{}",
        thread,
        msg,
        location.file(),
        location.line(),
      );
    } else {
      error_eprintln!("ERROR: '{}' panicked at '{}'", thread, msg);
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
