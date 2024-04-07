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
  path::{Path, PathBuf},
  process::Command,
};

use crate::{Extension, ExtensionOutput, ExtensionResult, Params, Value};
use log::{info, warn};
use thiserror::Error;

pub struct ScriptExtension {
  home_path: PathBuf,
  config_path: PathBuf,
  packages_path: PathBuf,
}

#[allow(clippy::new_without_default)]
impl ScriptExtension {
  pub fn new(config_path: &Path, home_path: &Path, packages_path: &Path) -> Self {
    Self {
      config_path: config_path.to_owned(),
      home_path: home_path.to_owned(),
      packages_path: packages_path.to_owned(),
    }
  }
}

impl Extension for ScriptExtension {
  fn name(&self) -> &str {
    "script"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    scope: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    if let Some(Value::Array(args)) = params.get("args") {
      let mut args: Vec<String> = args
        .iter()
        .filter_map(|arg| arg.as_string())
        .cloned()
        .collect();

      // Replace %HOME% with current user home directory to
      // create cross-platform paths. See issue #265
      // Also replace %CONFIG% and %PACKAGES% path. See issue #380
      for arg in &mut args {
        if arg.contains("%HOME%") {
          *arg = arg.replace("%HOME%", &self.home_path.to_string_lossy());
        }
        if arg.contains("%CONFIG%") {
          *arg = arg.replace("%CONFIG%", &self.config_path.to_string_lossy());
        }
        if arg.contains("%PACKAGES%") {
          *arg = arg.replace("%PACKAGES%", &self.packages_path.to_string_lossy());
        }

        // On Windows, correct paths separators
        if cfg!(target_os = "windows") {
          let path = PathBuf::from(&arg);
          if path.exists() {
            *arg = path.to_string_lossy().to_string();
          }
        }
      }

      let mut command = Command::new(&args[0]);
      command.env("CONFIG", self.config_path.to_string_lossy().to_string());
      for (key, value) in super::util::convert_to_env_variables(scope) {
        command.env(key, value);
      }

      // Set the OS-specific flags
      super::util::set_command_flags(&mut command);

      let output = if args.len() > 1 {
        command.args(&args[1..]).output()
      } else {
        command.output()
      };

      match output {
        Ok(output) => {
          let output_str = String::from_utf8_lossy(&output.stdout);
          let error_str = String::from_utf8_lossy(&output.stderr);

          let debug = params
            .get("debug")
            .and_then(|v| v.as_bool())
            .copied()
            .unwrap_or(false);

          if debug {
            info!("debug information for script> {:?}", args);
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
              "script command exited with code: {} and error: {}",
              output.status, error_str
            );

            if !ignore_error {
              return ExtensionResult::Error(
                ScriptExtensionError::ExecutionError(error_str.to_string()).into(),
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
          ScriptExtensionError::ExecutionFailed(args[0].to_string(), error.into()).into(),
        ),
      }
    } else {
      ExtensionResult::Error(ScriptExtensionError::MissingArgsParameter.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum ScriptExtensionError {
  #[error("missing 'args' parameter")]
  MissingArgsParameter,

  #[error("could not execute script: '`{0}`', error: '`{1}`'")]
  ExecutionFailed(String, anyhow::Error),

  #[error("script reported error: '`{0}`'")]
  ExecutionError(String),
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;
  #[cfg(not(target_os = "windows"))]
  use crate::Scope;

  fn get_extension() -> ScriptExtension {
    ScriptExtension::new(&PathBuf::new(), &PathBuf::new(), &PathBuf::new())
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn basic() {
    let extension = get_extension();

    let param = vec![(
      "args".to_string(),
      Value::Array(vec![
        Value::String("echo".to_string()),
        Value::String("hello world".to_string()),
      ]),
    )]
    .into_iter()
    .collect::<Params>();
    assert_eq!(
      extension
        .calculate(&crate::Context::default(), &HashMap::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("hello world".to_string())
    );
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn basic_no_trim() {
    let extension = get_extension();

    let param = vec![
      (
        "args".to_string(),
        Value::Array(vec![
          Value::String("echo".to_string()),
          Value::String("hello world".to_string()),
        ]),
      ),
      ("trim".to_string(), Value::Bool(false)),
    ]
    .into_iter()
    .collect::<Params>();
    if cfg!(target_os = "windows") {
      assert_eq!(
        extension
          .calculate(&crate::Context::default(), &HashMap::default(), &param)
          .into_success()
          .unwrap(),
        ExtensionOutput::Single("hello world\r\n".to_string())
      );
    } else {
      assert_eq!(
        extension
          .calculate(&crate::Context::default(), &HashMap::default(), &param)
          .into_success()
          .unwrap(),
        ExtensionOutput::Single("hello world\n".to_string())
      );
    }
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn var_injection() {
    let extension = get_extension();

    let param = vec![(
      "args".to_string(),
      Value::Array(vec![
        Value::String("sh".to_string()),
        Value::String("-c".to_string()),
        Value::String("echo $ESPANSO_VAR1".to_string()),
      ]),
    )]
    .into_iter()
    .collect::<Params>();
    let mut scope = Scope::new();
    scope.insert("var1", ExtensionOutput::Single("hello world".to_string()));
    assert_eq!(
      extension
        .calculate(&crate::Context::default(), &scope, &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("hello world".to_string())
    );
  }

  #[test]
  fn invalid_command() {
    let extension = get_extension();

    let param = vec![(
      "args".to_string(),
      Value::Array(vec![Value::String("nonexistentcommand".to_string())]),
    )]
    .into_iter()
    .collect::<Params>();
    assert!(matches!(
      extension.calculate(&crate::Context::default(), &HashMap::default(), &param),
      ExtensionResult::Error(_)
    ));
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn exit_error() {
    let extension = get_extension();

    let param = vec![(
      "args".to_string(),
      Value::Array(vec![
        Value::String("sh".to_string()),
        Value::String("-c".to_string()),
        Value::String("exit 1".to_string()),
      ]),
    )]
    .into_iter()
    .collect::<Params>();
    assert!(matches!(
      extension.calculate(&crate::Context::default(), &HashMap::default(), &param),
      ExtensionResult::Error(_)
    ));
  }

  #[test]
  #[cfg(not(target_os = "windows"))]
  fn ignore_error() {
    let extension = get_extension();

    let param = vec![
      (
        "args".to_string(),
        Value::Array(vec![
          Value::String("sh".to_string()),
          Value::String("-c".to_string()),
          Value::String("exit 1".to_string()),
        ]),
      ),
      ("ignore_error".to_string(), Value::Bool(true)),
    ]
    .into_iter()
    .collect::<Params>();
    assert_eq!(
      extension
        .calculate(&crate::Context::default(), &HashMap::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single(String::new())
    );
  }
}
