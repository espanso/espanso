/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use log::{error, warn};
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;
use std::process::Command;

pub struct ScriptExtension {}

impl ScriptExtension {
    pub fn new() -> ScriptExtension {
        ScriptExtension {}
    }
}

impl super::Extension for ScriptExtension {
    fn name(&self) -> String {
        String::from("script")
    }

    fn calculate(&self, params: &Mapping, user_args: &Vec<String>) -> Option<String> {
        let args = params.get(&Value::from("args"));
        if args.is_none() {
            warn!("No 'args' parameter specified for script variable");
            return None;
        }
        let args = args.unwrap().as_sequence();
        if let Some(args) = args {
            let mut str_args = args
                .iter()
                .map(|arg| arg.as_str().unwrap_or_default().to_string())
                .collect::<Vec<String>>();

            // The user has to enable argument concatenation explicitly
            let inject_args = params
                .get(&Value::from("inject_args"))
                .unwrap_or(&Value::from(false))
                .as_bool()
                .unwrap_or(false);
            if inject_args {
                str_args.extend(user_args.clone());
            }

            // Replace %HOME% with current user home directory to
            // create cross-platform paths. See issue #265
            let home_dir = dirs::home_dir().unwrap_or_default();
            str_args.iter_mut().for_each(|arg| {
                if arg.contains("%HOME%") {
                    *arg = arg.replace("%HOME%", &home_dir.to_string_lossy().to_string());
                }

                // On Windows, correct paths separators
                if cfg!(target_os = "windows") {
                    let path = PathBuf::from(&arg);
                    if path.exists() {
                        *arg = path.to_string_lossy().to_string()
                    }
                }
            });

            let mut command = Command::new(&str_args[0]);

            // Inject the $CONFIG variable
            command.env("CONFIG", crate::context::get_config_dir());

            let output = if str_args.len() > 1 {
                command.args(&str_args[1..]).output()
            } else {
                command.output()
            };

            match output {
                Ok(output) => {
                    let mut output_str =
                        String::from_utf8_lossy(output.stdout.as_slice()).to_string();
                    let error_str = String::from_utf8_lossy(output.stderr.as_slice());
                    let error_str = error_str.to_string();
                    let error_str = error_str.trim();

                    // Print stderror if present
                    if !error_str.is_empty() {
                        warn!("Script command reported error: \n{}", error_str);
                    }

                    // If specified, trim the output
                    let trim_opt = params.get(&Value::from("trim"));
                    let should_trim = if let Some(value) = trim_opt {
                        let val = value.as_bool();
                        val.unwrap_or(true)
                    } else {
                        true
                    };

                    if should_trim {
                        output_str = output_str.trim().to_owned()
                    }

                    return Some(output_str);
                }
                Err(e) => {
                    error!("Could not execute script '{:?}', error: {}", args, e);
                    return None;
                }
            }
        }

        error!("Could not execute script with args '{:?}'", args);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::Extension;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_script_basic() {
        let mut params = Mapping::new();
        params.insert(
            Value::from("args"),
            Value::from(vec!["echo", "hello world"]),
        );

        let extension = ScriptExtension::new();
        let output = extension.calculate(&params, &vec![]);

        assert!(output.is_some());
        assert_eq!(output.unwrap(), "hello world");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_script_basic_no_trim() {
        let mut params = Mapping::new();
        params.insert(
            Value::from("args"),
            Value::from(vec!["echo", "hello world"]),
        );
        params.insert(Value::from("trim"), Value::from(false));

        let extension = ScriptExtension::new();
        let output = extension.calculate(&params, &vec![]);

        assert!(output.is_some());
        assert_eq!(output.unwrap(), "hello world\n");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_script_inject_args_off() {
        let mut params = Mapping::new();
        params.insert(
            Value::from("args"),
            Value::from(vec!["echo", "hello world"]),
        );

        let extension = ScriptExtension::new();
        let output = extension.calculate(&params, &vec!["jon".to_owned()]);

        assert!(output.is_some());
        assert_eq!(output.unwrap(), "hello world");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_script_inject_args_on() {
        let mut params = Mapping::new();
        params.insert(
            Value::from("args"),
            Value::from(vec!["echo", "hello world"]),
        );
        params.insert(Value::from("inject_args"), Value::from(true));

        let extension = ScriptExtension::new();
        let output = extension.calculate(&params, &vec!["jon".to_owned()]);

        assert!(output.is_some());
        assert_eq!(output.unwrap(), "hello world jon");
    }
}
