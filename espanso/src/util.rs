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

#[cfg(target_os = "windows")]
pub fn set_command_flags(command: &mut Command) {
  use std::os::windows::process::CommandExt;
  // Avoid showing the shell window
  // See: https://github.com/federico-terzi/espanso/issues/249
  command.creation_flags(0x08000000);
}

#[cfg(not(target_os = "windows"))]
pub fn set_command_flags(_: &mut Command) {
  // NOOP on Linux and macOS
}