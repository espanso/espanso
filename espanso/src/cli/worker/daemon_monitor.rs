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

use std::{path::Path};

use anyhow::Result;
use crossbeam::channel::Sender;
use log::{error, info, warn};

use crate::lock::acquire_daemon_lock;

const DAEMON_STATUS_CHECK_INTERVAL: u64 = 1000;

pub fn initialize_and_spawn(runtime_dir: &Path, exit_notify: Sender<()>) -> Result<()> {
  let runtime_dir_clone = runtime_dir.to_path_buf();
  std::thread::Builder::new()
    .name("daemon-monitor".to_string())
    .spawn(move || {
      daemon_monitor_main(&runtime_dir_clone, exit_notify.clone());
    })?;

  Ok(())
}

fn daemon_monitor_main(runtime_dir: &Path, exit_notify: Sender<()>) {
  info!("monitoring the status of the daemon process");
  
  loop {
    let is_daemon_lock_free = {
      let lock = acquire_daemon_lock(runtime_dir);
      lock.is_some()
    };

    if is_daemon_lock_free {
      warn!("detected unexpected daemon termination, sending exit signal to the engine");
      if let Err(error) = exit_notify.send(()) {
        error!("unable to send daemon exit signal: {}", error);
      }
      break;
    }

    std::thread::sleep(std::time::Duration::from_millis(DAEMON_STATUS_CHECK_INTERVAL));
  }
}
