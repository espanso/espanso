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

use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

lazy_static! {
  static ref LAYOUT_EXTRACT_REGEX: Regex = Regex::new(r"^\[\('.*?', '(.*?)'\)").unwrap();
}

pub fn get_active_layout() -> Option<String> {
  match Command::new("gsettings")
    .arg("get")
    .arg("org.gnome.desktop.input-sources")
    .arg("mru-sources")
    .output()
  {
    Ok(output) => {
      let output_str = String::from_utf8_lossy(&output.stdout);
      let captures = LAYOUT_EXTRACT_REGEX.captures(&output_str)?;
      let layout = captures.get(1)?.as_str();
      Some(layout.to_string())
    }
    Err(err) => {
      error!(
        "unable to retrieve current keyboard layout with 'gsettings': {}",
        err
      );
      None
    }
  }
}

pub fn is_gnome() -> bool {
  let target_session_file = PathBuf::from("/usr/bin/gnome-session");
  target_session_file.exists()
}
