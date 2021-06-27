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
use thiserror::Error;

use crate::cli::PathsOverrides;

pub fn launch_daemon(paths_overrides: &PathsOverrides) -> Result<()> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["daemon", "--show-welcome"]);

  // We only inject the paths that were explicitly overrided because otherwise
  // the migration process might create some incompatibilities.
  // For example, after the migration the "packages" dir could have been
  // moved to a different one, and that might cause the daemon to crash
  // if we inject the old packages dir that was automatically resolved.
  if let Some(config_override) = &paths_overrides.config {
    command.env(
      "ESPANSO_CONFIG_DIR",
      config_override.to_string_lossy().to_string(),
    );
  }
  if let Some(packages_override) = &paths_overrides.packages {
    command.env(
      "ESPANSO_PACKAGE_DIR",
      packages_override.to_string_lossy().to_string(),
    );
  }
  if let Some(runtime_override) = &paths_overrides.runtime {
    command.env(
      "ESPANSO_RUNTIME_DIR",
      runtime_override.to_string_lossy().to_string(),
    );
  }

  let mut child = command.spawn()?;
  let result = child.wait()?;

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
