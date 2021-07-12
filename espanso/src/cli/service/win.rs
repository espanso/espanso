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
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs::create_dir_all};
use thiserror::Error;
use std::os::windows::process::CommandExt;

use crate::{error_eprintln, warn_eprintln};

pub fn register() -> Result<()> {
  let current_path = std::env::current_exe().expect("unable to get exec path");

  let shortcut_path = get_startup_shortcut_file()?;
  
  create_shortcut_target_file(&shortcut_path, &current_path, "launcher")
}

pub fn unregister() -> Result<()> {
  let shortcut_path = get_startup_shortcut_file()?;
  if !shortcut_path.is_file() {
    error_eprintln!("could not unregister espanso, as it's not registered");
    return Err(UnregisterError::EntryNotFound.into());
  }
  
  std::fs::remove_file(shortcut_path)?;

  Ok(())
}

#[derive(Error, Debug)]
pub enum UnregisterError {
  #[error("entry not found")]
  EntryNotFound,
}

pub fn is_registered() -> bool {
  match get_startup_shortcut_file() {
    Ok(shortcut_path) => {
      if !shortcut_path.is_file() {
        return false;
      }

      match get_shortcut_target_file(&shortcut_path) {
        Ok(target_path) => {
          // Check if the target file is the same as the current binary
          let current_path = std::env::current_exe().expect("unable to get exec path");

          if current_path != target_path {
            warn_eprintln!("WARNING: Espanso is already registered as a service, but it points to another executable,");
            warn_eprintln!("which can create some inconsistencies.");
            warn_eprintln!("To fix the problem, unregister and register espanso again with these commands:");
            warn_eprintln!("");
            warn_eprintln!("   espanso service unregister");
            warn_eprintln!("   espanso service register");
            warn_eprintln!("");
          }
          
          true
        },
        Err(err) => {
          error_eprintln!("unable to determine shortcut target path: {}", err);
          false
        },
      }
    }
    Err(err) => {
      error_eprintln!("could not locate shortcut file: {}", err);
      false
    }
  }
}

pub fn start_service() -> Result<()> {
  let current_path = std::env::current_exe().expect("unable to get exec path");

  Command::new(current_path)
    .args(&["launcher"])
    .creation_flags(0x08000008) // CREATE_NO_WINDOW + DETACHED_PROCESS
    .spawn()?;

  Ok(())
}

fn get_startup_dir() -> Result<PathBuf> {
  let home_dir = dirs::home_dir().expect("unable to obtain user's home folder");
  let app_data = home_dir.join("AppData");
  let roaming = app_data.join("Roaming");
  let microsoft = roaming.join("Microsoft");
  let windows = microsoft.join("Windows");
  let start_menu = windows.join("Start Menu");
  let programs = start_menu.join("Programs");
  let startup = programs.join("Startup");

  if !startup.is_dir() {
    create_dir_all(&startup)?;
  }

  Ok(startup)
}

fn get_startup_shortcut_file() -> Result<PathBuf> {
  let parent = get_startup_dir()?;
  Ok(parent.join("espanso.lnk"))
}

fn get_shortcut_target_file(shortcut_path: &Path) -> Result<PathBuf> {
  let output = Command::new("powershell")
            .arg("-c")
            .arg("$sh = New-Object -ComObject WScript.Shell; $target = $sh.CreateShortcut($env:TARGET_FILE_PATH).TargetPath; echo $target")
            .env("TARGET_FILE_PATH", shortcut_path.to_string_lossy().to_string())
            .output()?;

  if !output.status.success() {
    return Err(ShortcutError::PowershellNonZeroExitCode.into());
  }

  let raw_path = String::from_utf8_lossy(&output.stdout);
  let path = PathBuf::from(raw_path.trim().to_string());
  Ok(path)
}

fn create_shortcut_target_file(shortcut_path: &Path, target_path: &Path, arguments: &str) -> Result<()> {
  let output = Command::new("powershell")
            .arg("-c")
            .arg("$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut($env:SHORTCUT_PATH); $Shortcut.TargetPath = $env:TARGET_PATH; $Shortcut.Arguments = $env:TARGET_ARGS; $Shortcut.Save()")
            .env("SHORTCUT_PATH", shortcut_path.to_string_lossy().to_string())
            .env("TARGET_PATH", target_path.to_string_lossy().to_string())
            .env("TARGET_ARGS", arguments)
            .output()?;

  if !output.status.success() {
    return Err(ShortcutError::PowershellNonZeroExitCode.into());
  }

  Ok(())
}

#[derive(Error, Debug)]
pub enum ShortcutError {
  #[error("powershell exit with non-zero code")]
  PowershellNonZeroExitCode,
}
