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

use std::path::{Path, PathBuf};
use std::{fs::create_dir_all, io::ErrorKind};
use thiserror::Error;

use anyhow::{bail, Context, Result};
use log::{error, warn};

pub fn is_espanso_in_path() -> bool {
  PathBuf::from("/usr/local/bin/espanso").is_file()
}

pub fn add_espanso_to_path(prompt_when_necessary: bool) -> Result<()> {
  if crate::cli::util::is_subject_to_app_translocation_on_macos() {
    error_eprintln!("Unable to register Espanso to PATH, please move the Espanso.app bundle inside the /Applications directory to proceed.");
    error_eprintln!(
      "For more information, please see: https://github.com/espanso/espanso/issues/844"
    );
    bail!("macOS activated app-translocation on Espanso");
  }

  let target_link_dir = PathBuf::from("/usr/local/bin");
  let exec_path = std::env::current_exe()?;

  let target_link_path = target_link_dir.join("espanso");

  if !target_link_dir.is_dir() {
    warn!("/usr/local/bin folder does not exist, attempting to create one...");

    if let Err(error) = create_dir_all(&target_link_dir) {
      match error.kind() {
        ErrorKind::PermissionDenied if prompt_when_necessary => {
          warn!("target link file can't be accessed with current permissions, requesting elevated ones through AppleScript.");

          create_dir_and_link_with_applescript(&exec_path, &target_link_path)
            .context("unable to create link with AppleScript")?;

          return Ok(());
        }
        _other_error => {
          return Err(PathError::SymlinkError(error).into());
        }
      }
    }
  }

  if let Err(error) = std::os::unix::fs::symlink(&exec_path, &target_link_path) {
    match error.kind() {
      ErrorKind::PermissionDenied if prompt_when_necessary => {
        warn!("target link file can't be accessed with current permissions, requesting elevated ones through AppleScript.");

        create_dir_and_link_with_applescript(&exec_path, &target_link_path)
          .context("unable to create link with AppleScript")?;

        return Ok(());
      }
      _other_error => {
        return Err(PathError::SymlinkError(error).into());
      }
    }
  }

  Ok(())
}

fn create_dir_and_link_with_applescript(exec_path: &Path, target_link_path: &Path) -> Result<()> {
  let params = format!(
    r#"do shell script "mkdir -p /usr/local/bin && ln -sf '{}' '{}'" with administrator privileges"#,
    exec_path.to_string_lossy(),
    target_link_path.to_string_lossy(),
  );

  let mut child = std::process::Command::new("osascript")
    .args(["-e", &params])
    .spawn()?;

  let result = child.wait()?;

  if !result.success() {
    return Err(PathError::ElevationRequestFailure.into());
  }

  Ok(())
}

pub fn remove_espanso_from_path(prompt_when_necessary: bool) -> Result<()> {
  let target_link_path = PathBuf::from("/usr/local/bin/espanso");

  if std::fs::symlink_metadata(&target_link_path).is_err() {
    return Err(PathError::SymlinkNotFound.into());
  }

  if let Err(error) = std::fs::remove_file(&target_link_path) {
    match error.kind() {
      ErrorKind::PermissionDenied => {
        if prompt_when_necessary {
          warn!("target link file can't be accessed with current permissions, requesting elevated ones through AppleScript.");

          let params = format!(
            r#"do shell script "rm '{}'" with administrator privileges"#,
            target_link_path.to_string_lossy(),
          );

          let mut child = std::process::Command::new("osascript")
            .args(["-e", &params])
            .spawn()?;

          let result = child.wait()?;
          if !result.success() {
            return Err(PathError::ElevationRequestFailure.into());
          }
        } else {
          return Err(PathError::SymlinkError(error).into());
        }
      }
      _other_error => {
        return Err(PathError::SymlinkError(error).into());
      }
    }
  }

  Ok(())
}

#[derive(Error, Debug)]
pub enum PathError {
  #[error("symlink error: `{0}`")]
  SymlinkError(std::io::Error),

  #[error("elevation request failed")]
  ElevationRequestFailure,

  #[error("symlink does not exist, so there is nothing to remove.")]
  SymlinkNotFound,
}
