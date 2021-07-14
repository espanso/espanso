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

use super::{CliModule, CliModuleArgs};
use crate::{error_eprintln, exit_code::{SERVICE_ALREADY_RUNNING, SERVICE_FAILURE, SERVICE_NOT_REGISTERED, SERVICE_NOT_RUNNING, SERVICE_SUCCESS}, info_println, lock::acquire_worker_lock};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(not(target_os = "windows"))]
use unix::*;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
use win::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::*;

mod stop;

pub fn new() -> CliModule {
  CliModule {
    enable_logs: true,
    disable_logs_terminal_output: true,
    requires_paths: true,
    subcommand: "service".to_string(),
    log_mode: super::LogMode::AppendOnly,
    entry: service_main,
    ..Default::default()
  }
}

fn service_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths argument");
  let cli_args = args.cli_args.expect("missing cli_args");
  #[allow(unused_variables)]
  let paths_overrides = args.paths_overrides.expect("missing paths_overrides");

  if cli_args.subcommand_matches("register").is_some() {
    if let Err(err) = register() {
      error_eprintln!("unable to register service: {}", err);
      return SERVICE_FAILURE;
    } else {
      info_println!("service registered correctly!");
    }
  } else if cli_args.subcommand_matches("unregister").is_some() {
    if let Err(err) = unregister() {
      error_eprintln!("unable to unregister service: {}", err);
      return SERVICE_FAILURE;
    } else {
      info_println!("service unregistered correctly!");
    }
  } else if cli_args.subcommand_matches("check").is_some() {
    if is_registered() {
      info_println!("registered as a service");
    } else {
      error_eprintln!("not registered as a service");
      return SERVICE_NOT_REGISTERED;
    }
  } else if let Some(sub_args) = cli_args.subcommand_matches("start") {
    let lock_file = acquire_worker_lock(&paths.runtime);
    if lock_file.is_none() {
      error_eprintln!("espanso is already running!");
      return SERVICE_ALREADY_RUNNING;
    }
    drop(lock_file);

    if sub_args.is_present("unmanaged") && !cfg!(target_os = "windows") {
      // Unmanaged service
      #[cfg(unix)]
      {
        if let Err(err) = fork_daemon(&paths_overrides) {
          error_eprintln!("unable to start service (unmanaged): {}", err);
          return SERVICE_FAILURE;
        }
      }
      #[cfg(windows)]
      {
        unreachable!();
      }
    } else {
      // Managed service
      if let Err(err) = start_service() {
        error_eprintln!("unable to start service: {}", err);
        return SERVICE_FAILURE;
      } else {
        info_println!("espanso started correctly!");
      }
    }
  } else if cli_args.subcommand_matches("stop").is_some() {
    let lock_file = acquire_worker_lock(&paths.runtime);
    if lock_file.is_some() {
      error_eprintln!("espanso is not running!");
      return SERVICE_NOT_RUNNING;
    }
    drop(lock_file);

    if let Err(err) = stop::terminate_worker(&paths.runtime) {
      error_eprintln!("unable to stop espanso: {}", err);
      return SERVICE_FAILURE;
    }
  } 

  SERVICE_SUCCESS
}
