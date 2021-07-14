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
use log::{info, warn};
use std::process::Command;
use std::{fs::create_dir_all, process::ExitStatus};
use thiserror::Error;

// const SERVICE_PLIST_CONTENT: &str = include_str!("../../res/macos/com.federicoterzi.espanso.plist");
// const SERVICE_PLIST_FILE_NAME: &str = "com.federicoterzi.espanso.plist";

pub fn register() -> Result<()> {
  todo!();
}

#[derive(Error, Debug)]
pub enum RegisterError {
  #[error("launchctl load failed")]
  LaunchCtlLoadFailed,
}

pub fn unregister() -> Result<()> {
  todo!();
}

#[derive(Error, Debug)]
pub enum UnregisterError {
  #[error("plist entry not found")]
  PlistNotFound,
}

pub fn is_registered() -> bool {
  todo!();
}

pub fn start_service() -> Result<()> {
  todo!();
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
