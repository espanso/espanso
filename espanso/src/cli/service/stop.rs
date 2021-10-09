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
use std::{path::Path, time::Instant};
use thiserror::Error;

use espanso_ipc::IPCClient;

use crate::{
  ipc::{create_ipc_client_to_worker, IPCEvent},
  lock::acquire_worker_lock,
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

  let now = Instant::now();
  while now.elapsed() < std::time::Duration::from_secs(3) {
    let lock_file = acquire_worker_lock(runtime_dir);
    if lock_file.is_some() {
      return Ok(());
    }

    std::thread::sleep(std::time::Duration::from_millis(200));
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
