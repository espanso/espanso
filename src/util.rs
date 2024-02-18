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

use log::info;
use std::process::Command;
use sysinfo::{System, SystemExt};

#[cfg(target_os = "windows")]
pub fn set_command_flags(command: &mut Command) {
  use std::os::windows::process::CommandExt;
  // Avoid showing the shell window
  // See: https://github.com/espanso/espanso/issues/249
  command.creation_flags(0x0800_0000);
}

#[cfg(not(target_os = "windows"))]
pub fn set_command_flags(_: &mut Command) {
  // NOOP on Linux and macOS
}

#[cfg(target_os = "windows")]
pub fn attach_console() {
  // When using the windows subsystem we loose the terminal output.
  // Therefore we try to attach to the current console if available.
  unsafe { winapi::um::wincon::AttachConsole(0xFFFF_FFFF) };
}

#[cfg(not(target_os = "windows"))]
pub fn attach_console() {
  // Not necessary on Linux and macOS
}

pub fn log_system_info() {
  let sys = System::new();
  info!(
    "system info: {} v{} - kernel: {}",
    sys.name().unwrap_or_default(),
    sys.os_version().unwrap_or_default(),
    sys.kernel_version().unwrap_or_default()
  );
}
