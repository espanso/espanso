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
use log::{error, info};
use std::io::Write;
use std::process::Command;
use thiserror::Error;

pub struct ModuloManager {
  modulo_path: Option<String>,
}

impl ModuloManager {
  pub fn new() -> Self {
    let mut modulo_path: Option<String> = None;
    // Check if the `MODULO_PATH` env variable is configured
    if let Some(_modulo_path) = std::env::var_os("MODULO_PATH") {
      info!("using modulo from env variable at {:?}", _modulo_path);
      modulo_path = Some(_modulo_path.to_string_lossy().to_string())
    } else {
      // Check in the same directory of espanso
      if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
          let possible_path = parent.join("modulo");
          let possible_path = possible_path.to_string_lossy().to_string();

          if let Ok(output) = Command::new(&possible_path).arg("--version").output() {
            if output.status.success() {
              info!("using modulo from exe directory at {:?}", possible_path);
              modulo_path = Some(possible_path);
            }
          }
        }
      }

      // Otherwise check if present in the PATH
      if modulo_path.is_none() {
        if let Ok(output) = Command::new("modulo").arg("--version").output() {
          if output.status.success() {
            info!("using modulo executable found in PATH");
            modulo_path = Some("modulo".to_owned());
          }
        }
      }
    }

    Self { modulo_path }
  }

  // pub fn is_valid(&self) -> bool {
  //   self.modulo_path.is_some()
  // }

  // pub fn get_version(&self) -> Option<String> {
  //   if let Some(ref modulo_path) = self.modulo_path {
  //     if let Ok(output) = Command::new(modulo_path).arg("--version").output() {
  //       let version = String::from_utf8_lossy(&output.stdout);
  //       return Some(version.to_string());
  //     }
  //   }

  //   None
  // }

  pub fn invoke(&self, args: &[&str], body: &str) -> Result<String> {
    if let Some(modulo_path) = &self.modulo_path {
      let mut command = Command::new(modulo_path);
      command
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

      crate::util::set_command_flags(&mut command);

      let child = command.spawn();

      match child {
        Ok(mut child) => {
          if let Some(stdin) = child.stdin.as_mut() {
            match stdin.write_all(body.as_bytes()) {
              Ok(_) => {
                // Get the output
                match child.wait_with_output() {
                  Ok(child_output) => {
                    let output = String::from_utf8_lossy(&child_output.stdout);

                    // Check also if the program reports an error
                    let error = String::from_utf8_lossy(&child_output.stderr);
                    if !error.is_empty() {
                      error!("modulo reported an error: {}", error);
                    }

                    if !output.trim().is_empty() {
                      return Ok(output.to_string());
                    } else {
                      return Err(ModuloError::EmptyOutput.into());
                    }
                  }
                  Err(error) => {
                    return Err(ModuloError::Error(error).into());
                  }
                }
              }
              Err(error) => {
                return Err(ModuloError::Error(error).into());
              }
            }
          } else {
            return Err(ModuloError::StdinError.into());
          }
        }
        Err(error) => {
          return Err(ModuloError::Error(error).into());
        }
      }
    } else {
      return Err(ModuloError::MissingModulo.into());
    }
  }
}

#[derive(Error, Debug)]
pub enum ModuloError {
  #[error("attempt to invoke modulo even though it's not configured")]
  MissingModulo,

  #[error("modulo returned an empty output")]
  EmptyOutput,

  #[error("could not connect to modulo stdin")]
  StdinError,

  #[error("error occurred during modulo invocation")]
  Error(#[from] std::io::Error),
}
