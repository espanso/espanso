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

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::{Command, Output},
};

use crate::{Extension, ExtensionOutput, ExtensionResult, Params, Value};
use log::{info, warn};
use thiserror::Error;

pub enum Shell {
  Cmd,
  Powershell,
  WSL,
  WSL2,
  Bash,
  Sh,
}

impl Shell {
  fn execute_cmd(&self, cmd: &str, vars: &HashMap<String, String>) -> std::io::Result<Output> {
    let mut is_wsl = false;

    let mut command = match self {
      Shell::Cmd => {
        let mut command = Command::new("cmd");
        command.args(&["/C", &cmd]);
        command
      }
      Shell::Powershell => {
        let mut command = Command::new("powershell");
        command.args(&["-Command", &cmd]);
        command
      }
      Shell::WSL => {
        is_wsl = true;
        let mut command = Command::new("bash");
        command.args(&["-c", &cmd]);
        command
      }
      Shell::WSL2 => {
        is_wsl = true;
        let mut command = Command::new("wsl");
        command.args(&["bash", "-c", &cmd]);
        command
      }
      Shell::Bash => {
        let mut command = Command::new("bash");
        command.args(&["-c", &cmd]);
        command
      }
      Shell::Sh => {
        let mut command = Command::new("sh");
        command.args(&["-c", &cmd]);
        command
      }
    };

    // Set the OS-specific flags
    super::util::set_command_flags(&mut command);

    // Inject all the previous variables
    for (key, value) in vars.iter() {
      command.env(key, value);
    }

    // In WSL environment, we have to specify which ENV variables
    // should be passed to linux.
    // For more information: https://devblogs.microsoft.com/commandline/share-environment-vars-between-wsl-and-windows/
    if is_wsl {
      let mut tokens: Vec<&str> = Vec::new();
      tokens.push("CONFIG/p");

      // Add all the previous variables
      for (key, _) in vars.iter() {
        tokens.push(key);
      }

      let wsl_env = tokens.join(":");
      command.env("WSLENV", wsl_env);
    }

    command.output()
  }

  fn from_string(shell: &str) -> Option<Shell> {
    match shell {
      "cmd" => Some(Shell::Cmd),
      "powershell" => Some(Shell::Powershell),
      "wsl" => Some(Shell::WSL),
      "wsl2" => Some(Shell::WSL2),
      "bash" => Some(Shell::Bash),
      "sh" => Some(Shell::Sh),
      _ => None,
    }
  }
}

impl Default for Shell {
  fn default() -> Shell {
    if cfg!(target_os = "windows") {
      Shell::Powershell
    } else if cfg!(target_os = "macos") {
      Shell::Sh
    } else if cfg!(target_os = "linux") {
      Shell::Bash
    } else {
      panic!("invalid target os for shell")
    }
  }
}

pub struct ShellExtension {
  config_path: PathBuf,
}

#[allow(clippy::new_without_default)]
impl ShellExtension {
  pub fn new(config_path: &Path) -> Self {
    Self {
      config_path: config_path.to_owned(),
    }
  }
}

impl Extension for ShellExtension {
  fn name(&self) -> &str {
    "shell"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    scope: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    if let Some(Value::String(cmd)) = params.get("cmd") {
      let shell = if let Some(Value::String(shell_param)) = params.get("shell") {
        if let Some(shell) = Shell::from_string(shell_param) {
          shell
        } else {
          return ExtensionResult::Error(
            ShellExtensionError::InvalidShell(shell_param.to_string()).into(),
          );
        }
      } else {
        Shell::default()
      };

      let mut env_variables = super::util::convert_to_env_variables(&scope);
      env_variables.insert(
        "CONFIG".to_string(),
        self.config_path.to_string_lossy().to_string(),
      );

      match shell.execute_cmd(cmd, &env_variables) {
        Ok(output) => {
          let output_str = String::from_utf8_lossy(&output.stdout);
          let error_str = String::from_utf8_lossy(&output.stderr);

          let debug = params
            .get("debug")
            .and_then(|v| v.as_bool())
            .copied()
            .unwrap_or(false);

          if debug {
            info!("debug information for command> {}", cmd);
            info!("exit status: '{}'", output.status);
            info!("stdout: '{}'", output_str);
            info!("stderr: '{}'", error_str);
            info!("this debug information was shown because the 'debug' option is true.");
          }

          let ignore_error = params
            .get("ignore_error")
            .and_then(|v| v.as_bool())
            .copied()
            .unwrap_or(false);

          if !output.status.success() || !error_str.trim().is_empty() {
            warn!(
              "shell command exited with code: {} and error: {}",
              output.status, error_str
            );

            if !ignore_error {
              return ExtensionResult::Error(
                ShellExtensionError::ExecutionError(error_str.to_string()).into(),
              );
            }
          }

          let trim = params
            .get("trim")
            .and_then(|v| v.as_bool())
            .copied()
            .unwrap_or(true);

          let output = if trim {
            output_str.trim().to_owned()
          } else {
            output_str.to_string()
          };

          ExtensionResult::Success(ExtensionOutput::Single(output))
        }
        Err(error) => ExtensionResult::Error(
          ShellExtensionError::ExecutionFailed(cmd.to_string(), error.into()).into(),
        ),
      }
    } else {
      ExtensionResult::Error(ShellExtensionError::MissingCmdParameter.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum ShellExtensionError {
  #[error("missing 'cmd' parameter")]
  MissingCmdParameter,

  #[error("invalid shell: `{0}` is not a valid one")]
  InvalidShell(String),

  #[error("could not execute command: '`{0}`', error: '`{1}`'")]
  ExecutionFailed(String, anyhow::Error),

  #[error("command reported error: '`{0}`'")]
  ExecutionError(String),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Scope;
  use std::iter::FromIterator;

  #[test]
  fn shell_not_trimmed() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = Params::from_iter(
      vec![
        (
          "cmd".to_string(),
          Value::String("echo \"hello world\"".to_string()),
        ),
        ("trim".to_string(), Value::Bool(false)),
      ]
      .into_iter(),
    );
    if cfg!(target_os = "windows") {
      assert_eq!(
        extension
          .calculate(&Default::default(), &Default::default(), &param)
          .into_success()
          .unwrap(),
        ExtensionOutput::Single("hello world\r\n".to_string())
      );
    } else {
      assert_eq!(
        extension
          .calculate(&Default::default(), &Default::default(), &param)
          .into_success()
          .unwrap(),
        ExtensionOutput::Single("hello world\n".to_string())
      );
    }
  }

  #[test]
  fn shell_trimmed() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = Params::from_iter(
      vec![(
        "cmd".to_string(),
        Value::String("echo \"hello world\"".to_string()),
      )]
      .into_iter(),
    );
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("hello world".to_string())
    );
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn pipes() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = Params::from_iter(
      vec![(
        "cmd".to_string(),
        Value::String("echo \"hello world\" | cat".to_string()),
      )]
      .into_iter(),
    );
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("hello world".to_string())
    );
  }

  #[test]
  fn var_injection() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = if cfg!(not(target_os = "windows")) {
      Params::from_iter(
        vec![(
          "cmd".to_string(),
          Value::String("echo $ESPANSO_VAR1".to_string()),
        )]
        .into_iter(),
      )
    } else {
      Params::from_iter(
        vec![
          (
            "cmd".to_string(),
            Value::String("echo %ESPANSO_VAR1%".to_string()),
          ),
          ("shell".to_string(), Value::String("cmd".to_string())),
        ]
        .into_iter(),
      )
    };
    let mut scope = Scope::new();
    scope.insert("var1", ExtensionOutput::Single("hello world".to_string()));
    assert_eq!(
      extension
        .calculate(&Default::default(), &scope, &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("hello world".to_string())
    );
  }

  #[test]
  fn invalid_command() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = Params::from_iter(
      vec![(
        "cmd".to_string(),
        Value::String("nonexistentcommand".to_string()),
      )]
      .into_iter(),
    );
    assert!(matches!(
      extension.calculate(&Default::default(), &Default::default(), &param),
      ExtensionResult::Error(_)
    ));
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn exit_error() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param =
      Params::from_iter(vec![("cmd".to_string(), Value::String("exit 1".to_string()))].into_iter());
    assert!(matches!(
      extension.calculate(&Default::default(), &Default::default(), &param),
      ExtensionResult::Error(_)
    ));
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn ignore_error() {
    let extension = ShellExtension::new(&PathBuf::new());

    let param = Params::from_iter(
      vec![
        ("cmd".to_string(), Value::String("exit 1".to_string())),
        ("ignore_error".to_string(), Value::Bool(true)),
      ]
      .into_iter(),
    );
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("".to_string())
    );
  }
}
