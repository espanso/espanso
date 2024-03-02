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
use log::{error, warn};
use std::io::Write;
use std::process::Command;
use thiserror::Error;

pub struct ModuloManager {
    is_support_enabled: bool,
}

impl ModuloManager {
    pub fn new() -> Self {
        let is_support_enabled = if cfg!(feature = "modulo") {
            true
        } else {
            warn!("this version of espanso doesn't come with modulo support, so graphical features (such as Forms and Search) might not be available");
            false
        };

        Self { is_support_enabled }
    }

    pub fn spawn(&self, args: &[&str], body: &str) -> Result<()> {
        if self.is_support_enabled {
            let exec_path = std::env::current_exe().expect("unable to obtain current exec path");
            let mut command = Command::new(exec_path);
            let mut full_args = vec!["modulo"];
            full_args.extend(args);
            command
                .args(full_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            crate::util::set_command_flags(&mut command);

            let child = command.spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdin) = child.stdin.as_mut() {
                        match stdin.write_all(body.as_bytes()) {
                            Ok(()) => Ok(()),
                            Err(error) => Err(ModuloError::Error(error).into()),
                        }
                    } else {
                        Err(ModuloError::StdinError.into())
                    }
                }
                Err(error) => Err(ModuloError::Error(error).into()),
            }
        } else {
            Err(ModuloError::MissingModulo.into())
        }
    }

    pub fn invoke(&self, args: &[&str], body: &str) -> Result<String> {
        if self.is_support_enabled {
            let exec_path = std::env::current_exe().expect("unable to obtain current exec path");
            let mut command = Command::new(exec_path);
            let mut full_args = vec!["modulo"];
            full_args.extend(args);
            command
                .args(full_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            crate::util::set_command_flags(&mut command);

            let child = command.spawn();

            match child {
                Ok(mut child) => {
                    if let Some(stdin) = child.stdin.as_mut() {
                        match stdin.write_all(body.as_bytes()) {
                            Ok(()) => {
                                // Get the output
                                match child.wait_with_output() {
                                    Ok(child_output) => {
                                        let output = String::from_utf8_lossy(&child_output.stdout);

                                        // Check also if the program reports an error
                                        let error = String::from_utf8_lossy(&child_output.stderr);
                                        if !error.is_empty() {
                                            error!("modulo reported an error: {}", error);
                                        }

                                        if !child_output.status.success() {
                                            error!(
                                                "modulo exited with non-zero status code: {:?}",
                                                child_output.status.code()
                                            );
                                        }

                                        if output.trim().is_empty() {
                                            Err(ModuloError::EmptyOutput.into())
                                        } else {
                                            Ok(output.to_string())
                                        }
                                    }
                                    Err(error) => Err(ModuloError::Error(error).into()),
                                }
                            }
                            Err(error) => Err(ModuloError::Error(error).into()),
                        }
                    } else {
                        Err(ModuloError::StdinError.into())
                    }
                }
                Err(error) => Err(ModuloError::Error(error).into()),
            }
        } else {
            Err(ModuloError::MissingModulo.into())
        }
    }
}

#[derive(Error, Debug)]
pub enum ModuloError {
    #[error(
        "attempt to invoke modulo, but this version of espanso is not compiled with support for it"
    )]
    MissingModulo,

    #[error("modulo returned an empty output")]
    EmptyOutput,

    #[error("could not connect to modulo stdin")]
    StdinError,

    #[error("error occurred during modulo invocation")]
    Error(#[from] std::io::Error),
}
