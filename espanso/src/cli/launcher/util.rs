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

use std::{path::Path, process::Command};

use anyhow::{bail, Result};
use espanso_path::Paths;
use thiserror::Error;

use crate::{
  exit_code::{MIGRATE_CLEAN_FAILURE, MIGRATE_DIRTY_FAILURE},
  lock::acquire_legacy_lock,
  util::set_command_flags,
};

pub fn is_legacy_version_running(runtime_path: &Path) -> bool {
  let legacy_lock_file = acquire_legacy_lock(runtime_path);
  legacy_lock_file.is_none()
}

pub fn migrate_configuration(paths: &Paths) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
  command.args(["migrate", "--noconfirm"]);
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

  let mut child = command.spawn()?;
  let result = child.wait()?;

  if result.success() {
    Ok(())
  } else {
    match result.code() {
      Some(code) if code == MIGRATE_CLEAN_FAILURE => Err(MigrationError::Clean.into()),
      Some(code) if code == MIGRATE_DIRTY_FAILURE => Err(MigrationError::Dirty.into()),
      _ => Err(MigrationError::Unexpected.into()),
    }
  }
}

#[derive(Error, Debug)]
pub enum MigrationError {
  #[error("clean error")]
  Clean,

  #[error("dirty error")]
  Dirty,

  #[error("unexpected error")]
  Unexpected,
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
