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

use anyhow::Result;
use const_format::formatcp;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

use crate::{error_eprintln, info_println, warn_eprintln};

const LINUX_SERVICE_NAME: &str = "espanso";
const LINUX_SERVICE_CONTENT: &str = include_str!("../../res/linux/systemd.service");
#[allow(clippy::transmute_bytes_to_str)]
const LINUX_SERVICE_FILENAME: &str = formatcp!("{}.service", LINUX_SERVICE_NAME);

pub fn register() -> Result<()> {
  let service_file = get_service_file_path()?;

  if service_file.exists() {
    warn_eprintln!("service file already exists, this operation will overwrite it");
  }

  info_println!("creating service file in {:?}", service_file);
  let espanso_path = get_binary_path().expect("unable to get espanso executable path");

  let service_content = String::from(LINUX_SERVICE_CONTENT)
    .replace("{{{espanso_path}}}", &espanso_path.to_string_lossy());

  std::fs::write(service_file, service_content)?;

  info_println!("enabling systemd service");

  match Command::new("systemctl")
    .args(["--user", "enable", LINUX_SERVICE_NAME])
    .status()
  {
    Ok(status) => {
      if !status.success() {
        return Err(RegisterError::SystemdEnableFailed.into());
      }
    }
    Err(err) => {
      error_eprintln!("unable to call systemctl: {}", err);
      return Err(RegisterError::SystemdCallFailed(err.into()).into());
    }
  }

  Ok(())
}

#[derive(Error, Debug)]
pub enum RegisterError {
  #[error("systemctl command failed `{0}`")]
  SystemdCallFailed(anyhow::Error),

  #[error("systemctl enable failed")]
  SystemdEnableFailed,
}

pub fn unregister() -> Result<()> {
  let service_file = get_service_file_path()?;

  if !service_file.exists() {
    return Err(UnregisterError::ServiceFileNotFound.into());
  }

  info_println!("disabling espanso systemd service");

  match Command::new("systemctl")
    .args(["--user", "disable", LINUX_SERVICE_NAME])
    .status()
  {
    Ok(status) => {
      if !status.success() {
        return Err(UnregisterError::SystemdDisableFailed.into());
      }
    }
    Err(err) => {
      error_eprintln!("unable to call systemctl: {}", err);
      return Err(UnregisterError::SystemdCallFailed(err.into()).into());
    }
  }

  info_println!("deleting espanso systemd entry");
  std::fs::remove_file(service_file)?;

  Ok(())
}

#[derive(Error, Debug)]
pub enum UnregisterError {
  #[error("service file not found")]
  ServiceFileNotFound,

  #[error("failed to disable systemd service")]
  SystemdDisableFailed,

  #[error("systemctl command failed `{0}`")]
  SystemdCallFailed(anyhow::Error),
}

pub fn is_registered() -> bool {
  let res = Command::new("systemctl")
    .args(["--user", "is-enabled", LINUX_SERVICE_NAME])
    .output();
  if let Ok(output) = res {
    if !output.status.success() {
      return false;
    }

    // Make sure the systemd service points to the right binary
    lazy_static! {
      static ref EXEC_PATH_REGEX: Regex = Regex::new("ExecStart=(?P<path>.*?)\\s").unwrap();
    }

    match Command::new("systemctl")
      .args(["--user", "cat", LINUX_SERVICE_NAME])
      .output()
    {
      Ok(cmd_output) => {
        let output = String::from_utf8_lossy(cmd_output.stdout.as_slice());
        let output = output.trim();
        if cmd_output.status.success() {
          let caps = EXEC_PATH_REGEX.captures(output).unwrap();
          let path = caps.get(1).map_or("", |m| m.as_str());
          let espanso_path = get_binary_path().expect("unable to get espanso executable path");

          if espanso_path.to_string_lossy() == path {
            true
          } else {
            error_eprintln!("Espanso is registered as a systemd service, but it points to another binary location:");
            error_eprintln!("");
            error_eprintln!("  {}", path);
            error_eprintln!("");
            error_eprintln!("This could have been caused by an update that changed its location.");
            error_eprintln!("To solve the problem, please unregister and register espanso again with these commands:");
            error_eprintln!("");
            error_eprintln!("  espanso service unregister && espanso service register");
            error_eprintln!("");

            false
          }
        } else {
          error_eprintln!("systemctl command returned non-zero exit code");
          false
        }
      }
      Err(err) => {
        error_eprintln!("failed to execute systemctl: {}", err);
        false
      }
    }
  } else {
    false
  }
}

pub fn start_service() -> Result<()> {
  // Check if systemd is available in the system
  if let Ok(status) = Command::new("systemctl")
    .args(["--version"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .status()
  {
    if !status.success() {
      return Err(StartError::SystemctlNonZeroExitCode.into());
    }
  } else {
    error_eprintln!(
      "Systemd was not found in this system, which means espanso can't run in managed mode"
    );
    error_eprintln!("You can run it in unmanaged mode with `espanso service start --unmanaged`");
    error_eprintln!("");
    error_eprintln!(
      "NOTE: unmanaged mode means espanso does not rely on the system service manager"
    );
    error_eprintln!(
      "      to run, but as a result, you are in charge of starting/stopping espanso"
    );
    error_eprintln!("      when needed.");
    return Err(StartError::SystemdNotFound.into());
  }

  if !is_registered() {
    error_eprintln!("Unable to start espanso as a service as it's not been registered.");
    error_eprintln!("You can either register it first with `espanso service register` or");
    error_eprintln!("you can run it in unmanaged mode with `espanso service start --unmanaged`");
    error_eprintln!("");
    error_eprintln!(
      "NOTE: unmanaged mode means espanso does not rely on the system service manager"
    );
    error_eprintln!(
      "      to run, but as a result, you are in charge of starting/stopping espanso"
    );
    error_eprintln!("      when needed.");
    return Err(StartError::NotRegistered.into());
  }

  match Command::new("systemctl")
    .args(["--user", "start", LINUX_SERVICE_NAME])
    .status()
  {
    Ok(status) => {
      if !status.success() {
        return Err(StartError::SystemctlStartFailed.into());
      }
    }
    Err(err) => {
      return Err(StartError::SystemctlFailed(err.into()).into());
    }
  }

  Ok(())
}

#[derive(Error, Debug)]
pub enum StartError {
  #[error("not registered as a service")]
  NotRegistered,

  #[error("systemd not found")]
  SystemdNotFound,

  #[error("failed to start systemctl: `{0}`")]
  SystemctlFailed(anyhow::Error),

  #[error("systemctl non-zero exit code")]
  SystemctlNonZeroExitCode,

  #[error("failed to launch espanso service through systemctl")]
  SystemctlStartFailed,
}

fn get_service_file_dir() -> Result<PathBuf> {
  // User level systemd services should be placed in this directory:
  // $XDG_CONFIG_HOME/systemd/user/, usually: ~/.config/systemd/user/
  let config_dir = dirs::config_dir().expect("Could not get configuration directory");
  let systemd_dir = config_dir.join("systemd");
  let user_dir = systemd_dir.join("user");

  if !user_dir.is_dir() {
    create_dir_all(&user_dir)?;
  }

  Ok(user_dir)
}

fn get_service_file_path() -> Result<PathBuf> {
  Ok(get_service_file_dir()?.join(LINUX_SERVICE_FILENAME))
}

fn get_binary_path() -> Result<PathBuf> {
  // If executed as part of an AppImage, get the app image path instead of
  // the binary itself (which was extracted in a temp directory).
  if let Some(app_image_path) = std::env::var_os("APPIMAGE") {
    return Ok(PathBuf::from(app_image_path));
  }

  Ok(std::env::current_exe()?)
}
