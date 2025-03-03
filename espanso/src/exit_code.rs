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
pub const WORKER_EXIT_ALL_PROCESSES: i32 = 50;
pub const WORKER_RESTART: i32 = 51;
pub const WORKER_ERROR_EXIT_NO_CODE: i32 = 90;

pub const DAEMON_SUCCESS: i32 = 0;
pub const DAEMON_ALREADY_RUNNING: i32 = 1;
pub const DAEMON_GENERAL_ERROR: i32 = 2;
pub const DAEMON_FATAL_CONFIG_ERROR: i32 = 4;

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

pub const PACKAGE_SUCCESS: i32 = 0;
pub const PACKAGE_UNEXPECTED_FAILURE: i32 = 1;
pub const PACKAGE_INSTALL_FAILED: i32 = 2;
pub const PACKAGE_UNINSTALL_FAILED: i32 = 3;
pub const PACKAGE_LIST_FAILED: i32 = 4;
pub const PACKAGE_UPDATE_FAILED: i32 = 5;
pub const PACKAGE_UPDATE_PARTIAL_FAILURE: i32 = 6;

#[allow(dead_code)]
pub const UNEXPECTED_RUN_AS_ROOT: i32 = 42;

use crate::error_eprintln;

pub fn configure_custom_panic_hook(fail_exit_code: i32) {
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

    std::process::exit(fail_exit_code);
  }));
}
