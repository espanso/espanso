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

use anyhow::Result;
use thiserror::Error;
use espanso_path::Paths;

use crate::{exit_code::{MIGRATE_CLEAN_FAILURE, MIGRATE_DIRTY_FAILURE}, lock::acquire_legacy_lock};

pub fn is_legacy_version_running(runtime_path: &Path) -> bool {
  let legacy_lock_file = acquire_legacy_lock(runtime_path);
  if legacy_lock_file.is_none() {
    true
  } else {
    false
  }
}

pub fn migrate_configuration(paths: &Paths) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["migrate", "--noconfirm"]);
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
      Some(code) if code == MIGRATE_CLEAN_FAILURE => Err(MigrationError::CleanError.into()),
      Some(code) if code == MIGRATE_DIRTY_FAILURE=> Err(MigrationError::DirtyError.into()),
      _ => Err(MigrationError::UnexpectedError.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum MigrationError {
  #[error("clean error")]
  CleanError,

  #[error("dirty error")]
  DirtyError,

  #[error("unexpected error")]
  UnexpectedError,
}

pub fn add_espanso_to_path() -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["env-path", "--prompt", "register"]);

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