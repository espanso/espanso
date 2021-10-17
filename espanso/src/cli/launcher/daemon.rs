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

use std::process::Command;

use anyhow::Result;
use std::process::ExitStatus;
use thiserror::Error;

use crate::cli::util::CommandExt;
use crate::cli::PathsOverrides;

pub fn launch_daemon(paths_overrides: &PathsOverrides) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["daemon"]);
  command.with_paths_overrides(paths_overrides);

  let result = spawn_and_wait(command)?;

  if result.success() {
    Ok(())
  } else {
    Err(DaemonError::NonZeroExitCode.into())
  }
}

#[derive(Error, Debug)]
pub enum DaemonError {
  #[error("unexpected error, 'espanso daemon' returned a non-zero exit code.")]
  NonZeroExitCode,
}

#[cfg(not(target_os = "macos"))]
fn spawn_and_wait(mut command: Command) -> Result<ExitStatus> {
  let mut child = command.spawn()?;
  Ok(child.wait()?)
}

// On macOS, if we simply wait for the daemon process to terminate, the application will
// appear as "Not Responding" after a few seconds, even though it's working correctly.
// To avoid this undesirable behavior, we spawn an headless eventloop so that the
// launcher looks "alive", while waiting for the daemon
#[cfg(target_os = "macos")]
fn spawn_and_wait(mut command: Command) -> Result<ExitStatus> {
  let mut child = command.spawn()?;

  let result = std::thread::Builder::new()
    .name("daemon-monitor-thread".to_owned())
    .spawn(move || {
      let results = child.wait();

      espanso_mac_utils::exit_headless_eventloop();

      results
    })?;

  espanso_mac_utils::start_headless_eventloop();

  let thread_result = result.join().expect("unable to join daemon-monitor-thread");
  Ok(thread_result?)
}
