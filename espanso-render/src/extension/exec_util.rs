/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2022 Federico Terzi
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

pub enum MacShell {
  Bash,
  Nu,
  Pwsh,
  Sh,
  Zsh,
}

// Determine the PATH env variable value for macOS available inside a regular terminal session
pub fn determine_path_env_variable_override(explicit_shell: Option<MacShell>) -> Option<String> {
  if cfg!(not(target_os = "macos")) {
    return None;
  }
  let shell: MacShell = explicit_shell.or_else(determine_default_macos_shell)?;

  match shell {
    MacShell::Bash => {
      launch_command_and_get_output("bash", &["--login", "-c", "source ~/.bashrc; echo $PATH"])
    }
    MacShell::Nu => launch_command_and_get_output("nu", &["--login", "-c", "$env.PATH"]),
    MacShell::Pwsh => launch_command_and_get_output(
      "pwsh",
      &[
        "-Login",
        "-Command",
        "if(Test-Path \"$PROFILE\") { . \"$PROFILE\" }; Write-Host $env:PATH",
      ],
    ),
    MacShell::Sh => launch_command_and_get_output("sh", &["--login", "-c", "echo $PATH"]),
    MacShell::Zsh => {
      launch_command_and_get_output("zsh", &["--login", "-c", "source ~/.zshrc; echo $PATH"])
    }
  }
}

pub fn determine_default_macos_shell() -> Option<MacShell> {
  if cfg!(not(target_os = "macos")) {
    return None;
  }
  use lazy_static::lazy_static;
  use regex::Regex;
  use std::process::Command;

  let output = Command::new("sh")
    .args(["--login", "-c", "dscl . -read ~/ UserShell"])
    .output()
    .ok()?;

  lazy_static! {
    static ref EXTRACT_SHELL_REGEX: Regex =
      Regex::new(r"UserShell:\s(.*)$").expect("unable to generate regex to extract default shell");
  }

  if !output.status.success() {
    return None;
  }

  let output_str = String::from_utf8_lossy(&output.stdout);
  let captures = EXTRACT_SHELL_REGEX.captures(output_str.trim())?;

  let shell = captures.get(1)?.as_str().trim();

  if shell.ends_with("/bash") {
    Some(MacShell::Bash)
  } else if shell.ends_with("/nu") {
    Some(MacShell::Nu)
  } else if shell.ends_with("/pwsh") {
    Some(MacShell::Pwsh)
  } else if shell.ends_with("/sh") {
    Some(MacShell::Sh)
  } else if shell.ends_with("/zsh") {
    Some(MacShell::Zsh)
  } else {
    None
  }
}

fn launch_command_and_get_output(command: &str, args: &[&str]) -> Option<String> {
  use std::process::Command;

  let output = Command::new(command).args(args).output().ok()?;

  if !output.status.success() {
    return None;
  }

  let output_str = String::from_utf8_lossy(&output.stdout);
  Some(output_str.to_string())
}
