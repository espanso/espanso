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

use anyhow::{Error, Result};
use log::error;
use std::process::Command;
use std::{path::Path, time::Instant};
use sysinfo::{PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use thiserror::Error;

use espanso_ipc::IPCClient;

use crate::info_println;
use crate::{
  ipc::{create_ipc_client_to_worker, IPCEvent},
  lock::acquire_worker_lock,
  warn_eprintln,
};

pub fn terminate_worker(runtime_dir: &Path) -> Result<()> {
  match create_ipc_client_to_worker(runtime_dir) {
    Ok(mut worker_ipc) => {
      if let Err(err) = worker_ipc.send_async(IPCEvent::ExitAllProcesses) {
        error!(
          "unable to send termination signal to worker process: {}",
          err
        );
        return Err(StopError::IPCError(err).into());
      }
    }
    Err(err) => {
      error!("could not establish IPC connection with worker: {}", err);
      return Err(StopError::IPCError(err).into());
    }
  }

  if wait_for_worker_to_be_stopped(runtime_dir) {
    return Ok(());
  }

  warn_eprintln!(
    "unable to gracefully terminate espanso (timed-out), trying to force the termination..."
  );

  forcefully_terminate_espanso();

  if wait_for_worker_to_be_stopped(runtime_dir) {
    return Ok(());
  }

  Err(StopError::WorkerTimedOut.into())
}

#[derive(Error, Debug)]
pub enum StopError {
  #[error("worker timed out")]
  WorkerTimedOut,

  #[error("ipc error: `{0}`")]
  IPCError(Error),
}

fn wait_for_worker_to_be_stopped(runtime_dir: &Path) -> bool {
  let now = Instant::now();
  while now.elapsed() < std::time::Duration::from_secs(3) {
    let lock_file = acquire_worker_lock(runtime_dir);
    if lock_file.is_some() {
      return true;
    }

    std::thread::sleep(std::time::Duration::from_millis(200));
  }

  false
}

fn forcefully_terminate_espanso() {
  let mut sys =
    System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
  sys.refresh_processes_specifics(ProcessRefreshKind::new());

  let target_process_names = if cfg!(target_os = "windows") {
    vec!["espanso.exe", "espansod.exe"]
  } else {
    vec!["espanso"]
  };

  let current_pid = std::process::id();

  // We want to terminate all Espanso processes except this one
  for (pid, process) in sys.processes() {
    if target_process_names.contains(&process.name()) && pid.as_u32() != current_pid {
      let str_pid = pid.as_u32().to_string();
      info_println!("killing espanso process with PID: {}", str_pid);

      if cfg!(target_os = "windows") {
        let _ = Command::new("taskkill")
          .args(&["/pid", &str_pid, "/f"])
          .output();
      } else {
        let _ = Command::new("kill").args(&["-9", &str_pid]).output();
      }
    }
  }
}
