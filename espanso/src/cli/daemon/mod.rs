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

use std::{path::Path, process::Command, time::Instant};

use crossbeam::{
  channel::{unbounded, Sender},
  select,
};
use espanso_ipc::IPCClient;
use espanso_path::Paths;
use log::{error, info, warn};

use crate::{
  exit_code::{
    DAEMON_ALREADY_RUNNING, DAEMON_GENERAL_ERROR, DAEMON_LEGACY_ALREADY_RUNNING, DAEMON_SUCCESS,
    WORKER_EXIT_ALL_PROCESSES, WORKER_RESTART, WORKER_SUCCESS,
  },
  ipc::{create_ipc_client_to_worker, IPCEvent},
  lock::{acquire_daemon_lock, acquire_legacy_lock, acquire_worker_lock},
};

use super::{CliModule, CliModuleArgs};

mod ipc;
mod watcher;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: true,
    log_mode: super::LogMode::CleanAndAppend,
    subcommand: "daemon".to_string(),
    entry: daemon_main,
    ..Default::default()
  }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn daemon_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths in daemon main");
  let config_store = args
    .config_store
    .expect("missing config store in worker main");

  // Make sure only one instance of the daemon is running
  let lock_file = acquire_daemon_lock(&paths.runtime);
  if lock_file.is_none() {
    error!("daemon is already running!");
    return DAEMON_ALREADY_RUNNING;
  }

  let legacy_lock_file = acquire_legacy_lock(&paths.runtime);
  if legacy_lock_file.is_none() {
    // TODO: show a (blocking) alert message using modulo

    error!("an instance of legacy espanso is running, please terminate it, otherwise the new version cannot start");
    return DAEMON_LEGACY_ALREADY_RUNNING;
  }
  drop(legacy_lock_file);

  // TODO: we might need to check preconditions: accessibility on macOS, presence of binaries on Linux, etc

  info!("espanso version: {}", VERSION);
  // TODO: print os system and version? (with os_info crate)

  terminate_worker_if_already_running(&paths.runtime);

  let (exit_notify, exit_signal) = unbounded::<i32>();

  // TODO: register signals to terminate the worker if the daemon terminates

  spawn_worker(&paths, exit_notify.clone());

  ipc::initialize_and_spawn(&paths.runtime, exit_notify.clone())
    .expect("unable to initialize ipc server for daemon");

  let (watcher_notify, watcher_signal) = unbounded::<()>();
  
  if config_store.default().auto_restart() {
    watcher::initialize_and_spawn(&paths.config, watcher_notify)
      .expect("unable to initialize config watcher thread");
  }

  loop {
    select! {
      recv(watcher_signal) -> _ => {
        info!("configuration change detected, restarting worker process...");

        match create_ipc_client_to_worker(&paths.runtime) {
          Ok(mut worker_ipc) => {
            if let Err(err) = worker_ipc.send_async(IPCEvent::Exit) {
              error!(
                "unable to send termination signal to worker process: {}",
                err
              );
            }
          }
          Err(err) => {
            error!("could not establish IPC connection with worker: {}", err);
          }
        }

        // Wait until the worker process has terminated
        let start = Instant::now();
        let mut has_timed_out = true;
        while start.elapsed() < std::time::Duration::from_secs(30) {
          let lock_file = acquire_worker_lock(&paths.runtime);
          if lock_file.is_some() {
            has_timed_out = false;
            break;
          }

          std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if !has_timed_out {
          spawn_worker(&paths, exit_notify.clone());
        } else {
          error!("could not restart worker, as the exit process has timed out");
        }
      }
      recv(exit_signal) -> code => {
        match code {
          Ok(code) => {
            match code {
              WORKER_EXIT_ALL_PROCESSES => {
                info!("worker requested a general exit, quitting the daemon");
                break;
              }
              WORKER_RESTART => {
                info!("worker requested a restart, spawning a new one...");
                spawn_worker(&paths, exit_notify.clone());
              }
              _ => {
                error!("received unexpected exit code from worker {}, exiting", code);
                return code;
              }
            }
          },
          Err(err) => {
            error!("received error when unwrapping exit_code: {}", err);
            return DAEMON_GENERAL_ERROR;
          },
        }
      },
    }
  }

  DAEMON_SUCCESS
}

fn terminate_worker_if_already_running(runtime_dir: &Path) {
  let lock_file = acquire_worker_lock(&runtime_dir);
  if lock_file.is_some() {
    return;
  }

  warn!("a worker process is already running, sending termination signal...");

  match create_ipc_client_to_worker(runtime_dir) {
    Ok(mut worker_ipc) => {
      if let Err(err) = worker_ipc.send_async(IPCEvent::Exit) {
        error!(
          "unable to send termination signal to worker process: {}",
          err
        );
      }
    }
    Err(err) => {
      error!("could not establish IPC connection with worker: {}", err);
    }
  }

  let now = Instant::now();
  while now.elapsed() < std::time::Duration::from_secs(3) {
    let lock_file = acquire_worker_lock(runtime_dir);
    if lock_file.is_some() {
      return;
    }

    std::thread::sleep(std::time::Duration::from_millis(200));
  }

  panic!(
    "could not terminate worker process, please kill it manually, otherwise espanso won't start"
  )
}

fn spawn_worker(paths: &Paths, exit_notify: Sender<i32>) {
  info!("spawning the worker process...");

  let espanso_exe_path =
    std::env::current_exe().expect("unable to obtain espanso executable location");

  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["worker"]);
  command.env(
    "ESPANSO_CONFIG_DIR",
    paths.config.to_string_lossy().to_string(),
  );
  command.env(
    "ESPANSO_PACKAGE_DIR",
    paths.packages.to_string_lossy().to_string(),
  );
  command.env(
    "ESPANSO_RUNTIME_DIR",
    paths.runtime.to_string_lossy().to_string(),
  );

  // TODO: investigate if this is needed here, especially when invoking a form
  // // On windows, we need to spawn the process as "Detached"
  // #[cfg(target_os = "windows")]
  // {
  //   use std::os::windows::process::CommandExt;
  //   //command.creation_flags(0x08000008); // CREATE_NO_WINDOW + DETACHED_PROCESS
  // }

  let mut child = command.spawn().expect("unable to spawn worker process");

  // Create a monitor thread that will exit with the same non-zero code if
  // the worker thread exits
  std::thread::Builder::new()
    .name("worker-status-monitor".to_string())
    .spawn(move || {
      let result = child.wait();
      if let Ok(status) = result {
        if let Some(code) = status.code() {
          if code != WORKER_SUCCESS {
            exit_notify
              .send(code)
              .expect("unable to forward worker exit code");
          }
        }
      }
    })
    .expect("Unable to spawn worker monitor thread");
}
