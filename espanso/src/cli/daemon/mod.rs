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
  cli::util::{prevent_running_as_root_on_macos, CommandExt},
  common_flags::*,
  exit_code::{
    DAEMON_ALREADY_RUNNING, DAEMON_FATAL_CONFIG_ERROR, DAEMON_GENERAL_ERROR, DAEMON_SUCCESS,
    WORKER_ERROR_EXIT_NO_CODE, WORKER_EXIT_ALL_PROCESSES, WORKER_RESTART, WORKER_SUCCESS,
  },
  ipc::{create_ipc_client_to_worker, IPCEvent},
  lock::{acquire_daemon_lock, acquire_worker_lock},
  VERSION,
};

use super::{CliModule, CliModuleArgs, PathsOverrides};

mod ipc;
mod keyboard_layout_watcher;
mod troubleshoot;
mod watcher;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    enable_logs: true,
    log_mode: super::LogMode::CleanAndAppend,
    subcommand: "daemon".to_string(),
    entry: daemon_main,
    ..Default::default()
  }
}

fn daemon_main(args: CliModuleArgs) -> i32 {
  prevent_running_as_root_on_macos();

  let paths = args.paths.expect("missing paths in daemon main");
  let paths_overrides = args
    .paths_overrides
    .expect("missing paths_overrides in daemon main");

  // Make sure only one instance of the daemon is running
  let lock_file = acquire_daemon_lock(&paths.runtime);
  if lock_file.is_none() {
    error!("daemon is already running!");
    return DAEMON_ALREADY_RUNNING;
  }

  // TODO: we might need to check preconditions: accessibility on macOS, presence of binaries on Linux, etc

  // This variable holds the current troubleshooter guard.
  // When a guard is dropped, the troubleshooting GUI is killed
  // so this ensures that there is only one troubleshooter running
  // at a given time.
  //
  // it is currently unused
  // #[allow(unused_variables)]
  let mut _current_troubleshoot_guard = None;

  let (watcher_notify, watcher_signal) = unbounded::<()>();

  watcher::initialize_and_spawn(&paths.config, watcher_notify)
    .expect("unable to initialize config watcher thread");

  let (keyboard_layout_watcher_notify, keyboard_layout_watcher_signal) = unbounded::<()>();

  #[allow(clippy::redundant_clone)]
  // IMPORTANT: Here we clone the channel instead of simply passing it to avoid
  // dropping the channel immediately on those platforms that don't support the
  // layout watcher (currently Windows and macOS).
  // Otherwise, the select below would always return an error because the channel is closed.
  keyboard_layout_watcher::initialize_and_spawn(keyboard_layout_watcher_notify.clone()) // DON'T REMOVE THE CLONE!
    .expect("unable to initialize keyboard layout watcher thread");

  let config_store =
    match troubleshoot::load_config_or_troubleshoot_until_config_is_correct_or_abort(
      &paths,
      &paths_overrides,
      watcher_signal.clone(),
    ) {
      Ok((result, guard)) => {
        _current_troubleshoot_guard = guard;
        result.config_store
      }
      Err(err) => {
        error!("critical error while loading config: {}", err);
        return DAEMON_FATAL_CONFIG_ERROR;
      }
    };

  info!("espanso version: {}", VERSION);
  // TODO: print os system and version? (with os_info crate)

  terminate_worker_if_already_running(&paths.runtime);

  let (exit_notify, exit_signal) = unbounded::<i32>();

  // TODO: register signals to terminate the worker if the daemon terminates

  spawn_worker(&paths_overrides, exit_notify.clone(), None);

  ipc::initialize_and_spawn(&paths.runtime, exit_notify.clone())
    .expect("unable to initialize ipc server for daemon");

  loop {
    select! {
      recv(watcher_signal) -> _ => {
        if !config_store.default().auto_restart() {
          continue;
        }

        info!("configuration change detected, restarting worker process...");

        // Before killing the previous worker, we make sure there is no fatal error
        // in the configs.
        let should_restart_worker = match troubleshoot::load_config_or_troubleshoot(&paths, &paths_overrides) {
          troubleshoot::LoadResult::Correct(_) => {
            _current_troubleshoot_guard = None;
            true
          },
          troubleshoot::LoadResult::Warning(_, guard) => {
            _current_troubleshoot_guard = guard;
            true
          }
          troubleshoot::LoadResult::Fatal(guard) => {
            _current_troubleshoot_guard = Some(guard);
            error!("critical error while loading config, could not restart worker");
            false
          }
        };

        if should_restart_worker {
          restart_worker(&paths, &paths_overrides, exit_notify.clone(), Some(WORKER_START_REASON_CONFIG_CHANGED.to_string()));
        }
      }
      recv(keyboard_layout_watcher_signal) -> _ => {
        info!("keyboard layout change detected, restarting worker...");
        restart_worker(&paths, &paths_overrides, exit_notify.clone(), Some(WORKER_START_REASON_KEYBOARD_LAYOUT_CHANGED.to_string()));
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
                spawn_worker(&paths_overrides, exit_notify.clone(), Some(WORKER_START_REASON_MANUAL.to_string()));
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
  let lock_file = acquire_worker_lock(runtime_dir);
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

fn spawn_worker(
  paths_overrides: &PathsOverrides,
  exit_notify: Sender<i32>,
  start_reason: Option<String>,
) {
  info!("spawning the worker process...");

  let espanso_exe_path =
    std::env::current_exe().expect("unable to obtain espanso executable location");

  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());

  let mut args = vec!["worker", "--monitor-daemon"];
  if let Some(start_reason) = &start_reason {
    args.push("--start-reason");
    args.push(start_reason);
  }
  command.args(&args);
  command.with_paths_overrides(paths_overrides);

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
        } else {
          exit_notify
            .send(WORKER_ERROR_EXIT_NO_CODE)
            .expect("unable to forward worker exit code");
        }
      }
    })
    .expect("Unable to spawn worker monitor thread");
}

fn restart_worker(
  paths: &Paths,
  paths_overrides: &PathsOverrides,
  exit_notify: Sender<i32>,
  start_reason: Option<String>,
) {
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

  if has_timed_out {
    error!("could not restart worker, as the exit process has timed out");
  } else {
    spawn_worker(paths_overrides, exit_notify, start_reason);
  }
}
