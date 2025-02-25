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

use anyhow::{bail, Result};
use thiserror::Error;

use crate::util::set_command_flags;

pub fn is_legacy_version_running() -> bool {
  false
}

pub fn add_espanso_to_path() -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
  command.args(["env-path", "--prompt", "register"]);

  let mut child = command.spawn()?;
  let result = child.wait()?;

  if result.success() {
    Ok(())
  } else {
    Err(AddToPathError::NonZeroExitCode.into())
  }
}

#[derive(Error, Debug)]
pub enum AddToPathError {
  #[error("unexpected error, 'espanso env-path register' returned a non-zero exit code.")]
  NonZeroExitCode,
}

pub fn show_already_running_warning() -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
  command.args(["modulo", "welcome", "--already-running"]);

  let mut child = command.spawn()?;
  child.wait()?;
  Ok(())
}

pub fn configure_auto_start(auto_start: bool) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
  let mut args = vec!["service"];
  if auto_start {
    args.push("register");
  } else {
    args.push("unregister");
  }

  command.args(&args);
  set_command_flags(&mut command);

  let mut child = command.spawn()?;
  let result = child.wait()?;

  if result.success() {
    Ok(())
  } else {
    bail!("service registration returned non-zero exit code");
  }
}
