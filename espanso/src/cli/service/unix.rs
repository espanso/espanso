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

use crate::cli::util::CommandExt;
use anyhow::Result;
use thiserror::Error;

use crate::cli::PathsOverrides;

pub fn fork_daemon(paths_overrides: &PathsOverrides) -> Result<()> {
  let pid = unsafe { libc::fork() };
  if pid < 0 {
    return Err(ForkError::ForkFailed.into());
  }

  if pid > 0 {
    // Parent process
    return Ok(());
  }

  // Spawned process

  // Create a new SID for the child process
  let sid = unsafe { libc::setsid() };
  if sid < 0 {
    return Err(ForkError::SetSidFailed.into());
  }

  // Detach stdout and stderr
  let null_path = std::ffi::CString::new("/dev/null").expect("CString unwrap failed");
  unsafe {
    let fd = libc::open(null_path.as_ptr(), libc::O_RDWR, 0);
    if fd != -1 {
      libc::dup2(fd, libc::STDIN_FILENO);
      libc::dup2(fd, libc::STDOUT_FILENO);
      libc::dup2(fd, libc::STDERR_FILENO);
    }
  };

  spawn_launcher(paths_overrides)
}

pub fn spawn_launcher(paths_overrides: &PathsOverrides) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = std::process::Command::new(espanso_exe_path.to_string_lossy().to_string());
  command.args(["launcher"]);
  command.with_paths_overrides(paths_overrides);

  let mut child = command.spawn()?;
  let result = child.wait()?;

  if result.success() {
    Ok(())
  } else {
    Err(ForkError::LauncherSpawnFailure.into())
  }
}

#[derive(Error, Debug)]
pub enum ForkError {
  #[error("unable to fork")]
  ForkFailed,

  #[error("setsid failed")]
  SetSidFailed,

  #[error("launcher spawn failure")]
  LauncherSpawnFailure,
}
