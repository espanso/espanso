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

use anyhow::{bail, Result};
use std::io::Write;
use std::{
  collections::HashSet,
  process::{Command, Stdio},
};

const BLUR_CHROME_WINDOWS_SCRIPT: &str =
  include_str!("../../res/macos/scripts/blur_chrome_windows.scpt");

const GET_RUNNING_APPS_SCRIPT: &str = include_str!("../../res/macos/scripts/get_running_apps.scpt");

const FOCUS_BITWARDEN_SCRIPT: &str = include_str!("../../res/macos/scripts/focus_bitwarden.scpt");

const SECURE_INPUT_ASK_LOCK_SCREEN_SCRIPT: &str =
  include_str!("../../res/macos/scripts/secure_input_ask_lock_screen.scpt");

const SUCCESS_DIALOG_SCRIPT: &str =
  include_str!("../../res/macos/scripts/secure_input_disabled_dialog.scpt");

pub fn run_secure_input_workaround() -> Result<()> {
  if espanso_mac_utils::get_secure_input_pid().is_none() {
    println!("secure input is not active, no workaround needed");
    return Ok(());
  }

  execute_secure_input_workaround()?;
  let _ = run_apple_script(SUCCESS_DIALOG_SCRIPT);
  Ok(())
}

fn execute_secure_input_workaround() -> Result<()> {
  println!(
    "Secure input is enabled. Our guess is that it was activated by '{}',",
    espanso_mac_utils::get_secure_input_application()
      .map(|entry| entry.0)
      .unwrap_or_default()
  );
  println!("so restarting that application could solve the problem.");
  println!();
  println!("Unfortunately, that guess might be wrong if SecureInput was activated by");
  println!("the application while in the background.");
  println!();
  println!("This workaround will attempt to execute a series of known actions that often");
  println!("help in disabling secure input.");

  let running_apps = get_running_apps()?;

  if running_apps.contains("com.google.Chrome") {
    println!("-> Running chrome defocusing workaround");
    if let Err(err) = run_apple_script(BLUR_CHROME_WINDOWS_SCRIPT) {
      eprintln!("unable to run chrome defocusing workaround: {err}");
    }

    if espanso_mac_utils::get_secure_input_pid().is_none() {
      return Ok(());
    }
  }

  if running_apps.contains("com.bitwarden.desktop") {
    println!("-> Focusing/Defocusing on Bitwarden");
    if let Err(err) = run_apple_script(FOCUS_BITWARDEN_SCRIPT) {
      eprintln!("unable to run bitwarden defocusing workaround: {err}");
    }

    if espanso_mac_utils::get_secure_input_pid().is_none() {
      return Ok(());
    }
  }

  // Ask the user if he wants to try locking the screen
  if run_apple_script(SECURE_INPUT_ASK_LOCK_SCREEN_SCRIPT)?.trim() == "yes" {
    if let Err(err) = lock_screen() {
      eprintln!("failed to lock the screen: {err}");
    }
  }

  if espanso_mac_utils::get_secure_input_pid().is_some() {
    bail!("failed to release secure input");
  }

  Ok(())
}

fn run_apple_script(script: &str) -> Result<String> {
  let mut child = Command::new("osascript")
    .arg("-")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

  let child_stdin = child.stdin.as_mut().unwrap();
  child_stdin.write_all(script.as_bytes())?;

  let output = child.wait_with_output()?;
  let stdout = String::from_utf8_lossy(&output.stdout);
  Ok(stdout.to_string())
}

fn lock_screen() -> Result<()> {
  let mut child = Command::new("osascript")
    .arg("-e")
    .arg(r#"tell application "System Events" to keystroke "q" using {command down,control down}"#)
    .spawn()?;

  child.wait()?;
  Ok(())
}

fn get_running_apps() -> Result<HashSet<String>> {
  let apps_raw = run_apple_script(GET_RUNNING_APPS_SCRIPT)?;
  let mut apps = HashSet::new();
  for app in apps_raw.trim().split(", ") {
    apps.insert(app.to_string());
  }

  Ok(apps)
}
