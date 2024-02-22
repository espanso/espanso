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
use winreg::enums::*;
use winreg::RegKey;

pub fn add_espanso_to_path(_: bool) -> Result<()> {
  let mut paths = read_paths();
  let exe_path = std::env::current_exe().expect("unable to obtain exec path");
  let parent_path = exe_path.parent().expect("unable to obtain parent path");

  // Add espanso to path list
  paths.push(parent_path.to_string_lossy().to_string());

  let path = paths.join(";");
  write_user_path_value(path)?;

  // Send broadcast to let other applications know that the env variables have changed
  send_change_broadcast();

  Ok(())
}

pub fn remove_espanso_from_path(_: bool) -> Result<()> {
  let paths = read_paths();
  let paths_without_espanso: Vec<String> = paths
    .into_iter()
    .filter(|path| !path.contains("espanso"))
    .collect();
  let path = paths_without_espanso.join(";");
  write_user_path_value(path)
}

fn read_user_path_value() -> Result<String> {
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let env = hkcu.open_subkey("Environment")?;
  let path: String = env.get_value("Path")?;
  Ok(path)
}

fn read_paths() -> Vec<String> {
  let path_value = read_user_path_value().unwrap_or_default();
  let paths = path_value.split(';');
  paths.map(String::from).collect()
}

fn write_user_path_value(value: String) -> Result<()> {
  let hkcu = RegKey::predef(HKEY_CURRENT_USER);
  let env = hkcu.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
  env.set_value("Path", &value)?;
  Ok(())
}

fn send_change_broadcast() {
  use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE};

  let wide_string = widestring::WideString::from("Environment".to_string());

  unsafe {
    let mut res: usize = 0;
    SendMessageTimeoutW(
      HWND_BROADCAST,
      WM_SETTINGCHANGE,
      0,
      wide_string.as_ptr() as isize,
      2,
      50,
      &mut res,
    );
  }
}
