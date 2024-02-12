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

use log::error;

pub fn get_active_layout() -> Option<String> {
  match Command::new("setxkbmap").arg("-query").output() {
    Ok(output) => {
      let output_str = String::from_utf8_lossy(&output.stdout);
      let layout_line = output_str.lines().find(|line| line.contains("layout:"))?;
      let layout_raw: Vec<&str> = layout_line.split("layout:").collect();
      Some(layout_raw.get(1)?.trim().to_string())
    }
    Err(err) => {
      error!(
        "unable to retrieve current keyboard layout with 'setxkbmap': {}",
        err
      );
      None
    }
  }
}
