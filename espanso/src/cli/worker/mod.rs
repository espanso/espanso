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

use crossbeam::channel::unbounded;
use log::{error, info};

use crate::{
  engine::event::ExitMode,
  exit_code::{
    WORKER_ALREADY_RUNNING, WORKER_EXIT_ALL_PROCESSES, WORKER_GENERAL_ERROR,
    WORKER_LEGACY_ALREADY_RUNNING, WORKER_RESTART, WORKER_SUCCESS,
  },
  lock::{acquire_legacy_lock, acquire_worker_lock},
};

use self::ui::util::convert_icon_paths_to_tray_vec;

use super::{CliModule, CliModuleArgs};

mod config;
mod daemon_monitor;
mod engine;
mod ipc;
mod match_cache;
mod ui;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: true,
    log_mode: super::LogMode::AppendOnly,
    subcommand: "worker".to_string(),
    entry: worker_main,
    ..Default::default()
  }
}

fn worker_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths in worker main");
  let cli_args = args.cli_args.expect("missing cli_args in worker main");

  // Avoid running multiple worker instances
  let lock_file = acquire_worker_lock(&paths.runtime);
  if lock_file.is_none() {
    error!("worker is already running!");
    return WORKER_ALREADY_RUNNING;
  }

  let legacy_lock_file = acquire_legacy_lock(&paths.runtime);
  if legacy_lock_file.is_none() {
    error!("an instance of legacy espanso is running, please terminate it, otherwise the new version cannot start");
    return WORKER_LEGACY_ALREADY_RUNNING;
  }
  drop(legacy_lock_file);

  let config_store = args
    .config_store
    .expect("missing config store in worker main");
  let match_store = args
    .match_store
    .expect("missing match store in worker main");

  // TODO: show config loading errors in a GUI, if any

  let icon_paths =
    crate::icon::load_icon_paths(&paths.runtime).expect("unable to initialize icons");

  let (remote, mut eventloop) = espanso_ui::create_ui(espanso_ui::UIOptions {
    // TODO: handle show icon
    icon_paths: convert_icon_paths_to_tray_vec(&icon_paths),
    notification_icon_path: icon_paths
      .logo
      .as_ref()
      .map(|path| path.to_string_lossy().to_string()),
    ..Default::default()
  })
  .expect("unable to create tray icon UI module");

  eventloop
    .initialize()
    .expect("unable to initialize UI module");

  let (engine_exit_notify, engine_exit_receiver) = unbounded();
  let (engine_ui_event_sender, engine_ui_event_receiver) = unbounded();

  // Initialize the engine on another thread and start it
  let engine_handle = engine::initialize_and_spawn(
    paths.clone(),
    config_store,
    match_store,
    remote,
    engine_exit_receiver,
    engine_ui_event_receiver,
  )
  .expect("unable to initialize engine");

  // Setup the IPC server
  ipc::initialize_and_spawn(&paths.runtime, engine_exit_notify.clone())
    .expect("unable to initialize IPC server");

  // If specified, automatically monitor the daemon status and
  // terminate the worker if the daemon terminates
  // This is needed to avoid "dangling" worker processes
  // if the daemon crashes or is forcefully terminated.
  if cli_args.is_present("monitor-daemon") {
    daemon_monitor::initialize_and_spawn(&paths.runtime, engine_exit_notify.clone())
      .expect("unable to initialize daemon monitor thread");
  }

  eventloop
    .run(Box::new(move |event| {
      if let Err(error) = engine_ui_event_sender.send(event) {
        error!("unable to send UIEvent to engine: {}", error);
      }
    }))
    .expect("unable to run main eventloop");

  info!("waiting for engine exit mode...");
  match engine_handle.join() {
    Ok(mode) => match mode {
      ExitMode::Exit => {
        info!("exiting worker process...");
        return WORKER_SUCCESS;
      }
      ExitMode::ExitAllProcesses => {
        info!("exiting worker process and daemon...");
        return WORKER_EXIT_ALL_PROCESSES;
      }
      ExitMode::RestartWorker => {
        info!("exiting worker (to be restarted)");
        return WORKER_RESTART;
      }
    },
    Err(err) => {
      error!("unable to read engine exit mode: {:?}", err);
      return WORKER_GENERAL_ERROR;
    }
  }
}
