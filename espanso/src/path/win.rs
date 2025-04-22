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
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
  SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};
use winreg::enums::*;
use winreg::RegKey;

pub fn is_espanso_in_path() -> bool {
  read_paths().iter().any(|path| path.contains("espanso"))
}

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
  let wide_string = widestring::WideString::from("Environment".to_string());

  unsafe {
    // a null ptr
    let res = std::ptr::null_mut::<usize>();

    // docs: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw
    SendMessageTimeoutW(
      HWND_BROADCAST,
      // WM_SETTINGCHANGE, WPARM and LPARM docs:
      // https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange
      WM_SETTINGCHANGE,
      WPARAM(0),
      LPARAM(*wide_string.as_ptr() as isize),
      SMTO_ABORTIFHUNG,
      50,
      Some(res),
    );
  }
}
