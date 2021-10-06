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

use crate::warn_eprintln;
use espanso_modulo::wizard::DetectedOS;

pub fn is_wrong_edition() -> (bool, DetectedOS) {
  if !cfg!(target_os = "linux") {
    return (false, DetectedOS::Unknown);
  }

  match get_session_type().as_deref() {
    Some("x11") if cfg!(feature = "wayland") => return (true, DetectedOS::X11),
    Some("wayland") if !cfg!(feature = "wayland") => return (true, DetectedOS::Wayland),
    None => {
      warn_eprintln!("could not automatically determine the session type (X11/Wayland), so make sure you have the correct espanso version!");
    }
    _ => {}
  }

  (false, DetectedOS::Unknown)
}

fn get_session_type() -> Option<String> {
  let output = std::process::Command::new("sh")
    .arg("-c")
    .arg("loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type")
    .output()
    .ok()?;

  if !output.status.success() {
    return None;
  }

  let raw_session_type = String::from_utf8_lossy(&output.stdout);
  let raw_session_type = raw_session_type.trim();
  if !raw_session_type.contains("Type=") {
    return None;
  }

  let session_type: Option<&str> = raw_session_type.split('=').into_iter().last();
  session_type.map(String::from)
}
