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

use anyhow::{bail, Result};
use log::{error, info, warn};
use std::process::Command;
use std::{fs::create_dir_all, process::ExitStatus};
use thiserror::Error;

use crate::cli::util::prevent_running_as_root_on_macos;
use crate::error_eprintln;

#[cfg(target_os = "macos")]
const SERVICE_PLIST_CONTENT: &str = include_str!("../../res/macos/com.federicoterzi.espanso.plist");
#[cfg(target_os = "macos")]
const SERVICE_PLIST_FILE_NAME: &str = "com.federicoterzi.espanso.plist";

pub fn register() -> Result<()> {
  prevent_running_as_root_on_macos();

  if crate::cli::util::is_subject_to_app_translocation_on_macos() {
    error_eprintln!("Unable to register Espanso as service, please move the Espanso.app bundle inside the /Applications directory to proceed.");
    error_eprintln!(
      "For more information, please see: https://github.com/espanso/espanso/issues/844"
    );
    bail!("macOS activated app-translocation on Espanso");
  }

  let home_dir = dirs::home_dir().expect("could not get user home directory");
  let library_dir = home_dir.join("Library");
  let agents_dir = library_dir.join("LaunchAgents");

  // Make sure agents directory exists
  if !agents_dir.exists() {
    create_dir_all(agents_dir.clone())?;
  }

  let plist_file = agents_dir.join(SERVICE_PLIST_FILE_NAME);
  if !plist_file.exists() {
    info!(
      "creating LaunchAgents entry: {}",
      plist_file.to_str().unwrap_or_default()
    );

    let espanso_path = std::env::current_exe()?;
    info!(
      "entry will point to: {}",
      espanso_path.to_str().unwrap_or_default()
    );

    let plist_content = String::from(SERVICE_PLIST_CONTENT).replace(
      "{{{espanso_path}}}",
      espanso_path.to_str().unwrap_or_default(),
    );

    // Copy the user PATH variable and inject it in the Plist file so that
    // it gets loaded by Launchd.
    // To see why this is necessary: https://github.com/espanso/espanso/issues/233
    let user_path = std::env::var("PATH").unwrap_or_else(|_| String::new());
    let plist_content = plist_content.replace("{{{PATH}}}", &user_path);

    std::fs::write(plist_file.clone(), plist_content).expect("Unable to write plist file");
  }

  info!("reloading espanso launchctl entry");

  if let Err(err) = Command::new("launchctl")
    .args(["unload", "-w", plist_file.to_str().unwrap_or_default()])
    .output()
  {
    warn!("unload command failed: {}", err);
  }

  let res = Command::new("launchctl")
    .args(["load", "-w", plist_file.to_str().unwrap_or_default()])
    .status();

  if let Ok(status) = res {
    if status.success() {
      return Ok(());
    }
  }

  Err(RegisterError::LaunchCtlLoadFailed.into())
}

#[derive(Error, Debug)]
pub enum RegisterError {
  #[error("launchctl load failed")]
  LaunchCtlLoadFailed,
}

pub fn unregister() -> Result<()> {
  prevent_running_as_root_on_macos();

  let home_dir = dirs::home_dir().expect("could not get user home directory");
  let library_dir = home_dir.join("Library");
  let agents_dir = library_dir.join("LaunchAgents");

  let plist_file = agents_dir.join(SERVICE_PLIST_FILE_NAME);
  if plist_file.exists() {
    let _res = Command::new("launchctl")
      .args(["unload", "-w", plist_file.to_str().unwrap_or_default()])
      .output();

    std::fs::remove_file(&plist_file)?;

    Ok(())
  } else {
    Err(UnregisterError::PlistNotFound.into())
  }
}

#[derive(Error, Debug)]
pub enum UnregisterError {
  #[error("plist entry not found")]
  PlistNotFound,
}

pub fn is_registered() -> bool {
  let home_dir = dirs::home_dir().expect("could not get user home directory");
  let library_dir = home_dir.join("Library");
  let agents_dir = library_dir.join("LaunchAgents");
  let plist_file = agents_dir.join(SERVICE_PLIST_FILE_NAME);
  plist_file.is_file()
}

pub fn start_service() -> Result<()> {
  if !is_registered() {
    eprintln!("Unable to start espanso as a service as it's not been registered.");
    eprintln!("You can either register it first with `espanso service register` or");
    eprintln!("you can run it in unmanaged mode with `espanso service start --unmanaged`");
    eprintln!();
    eprintln!("NOTE: unmanaged mode means espanso does not rely on the system service manager");
    eprintln!("      to run, but as a result, you are in charge of starting/stopping espanso");
    eprintln!("      when needed.");
    return Err(StartError::NotRegistered.into());
  }

  let res = Command::new("launchctl")
    .args(["start", "com.federicoterzi.espanso"])
    .status();

  if let Ok(status) = res {
    if status.success() {
      Ok(())
    } else {
      Err(StartError::LaunchCtlNonZeroExit(status).into())
    }
  } else {
    Err(StartError::LaunchCtlFailure.into())
  }
}

#[derive(Error, Debug)]
pub enum StartError {
  #[error("not registered as a service")]
  NotRegistered,

  #[error("launchctl failed to run")]
  LaunchCtlFailure,

  #[error("launchctl exited with non-zero code `{0}`")]
  LaunchCtlNonZeroExit(ExitStatus),
}
