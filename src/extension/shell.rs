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

use crate::extension::ExtensionResult;
use log::{error, info, warn};
use regex::{Captures, Regex};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::process::{Command, Output};

lazy_static! {
    static ref UNIX_POS_ARG_REGEX: Regex = Regex::new("\\$(?P<pos>\\d+)").unwrap();
    static ref WIN_POS_ARG_REGEX: Regex = Regex::new("%(?P<pos>\\d+)").unwrap();
}

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
        crate::utils::set_command_flags(&mut command);

        // Inject the $CONFIG variable
        command.env("CONFIG", crate::context::get_config_dir());

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

    fn get_arg_regex(&self) -> &Regex {
        let regex = match self {
            Shell::Cmd | Shell::Powershell => {
                &*WIN_POS_ARG_REGEX
            }
            _ => {
                &*UNIX_POS_ARG_REGEX
            }
        };
        regex 
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

pub struct ShellExtension {}

impl ShellExtension {
    pub fn new() -> ShellExtension {
        ShellExtension {}
    }
}

impl super::Extension for ShellExtension {
    fn name(&self) -> String {
        String::from("shell")
    }

    fn calculate(
        &self,
        params: &Mapping,
        args: &Vec<String>,
        vars: &HashMap<String, ExtensionResult>,
    ) -> super::ExtensionOut {
        let cmd = params.get(&Value::from("cmd"));
        if cmd.is_none() {
            warn!("No 'cmd' parameter specified for shell variable");
            return Err(super::ExtensionError::Internal);
        }

        let inject_args = params
            .get(&Value::from("inject_args"))
            .unwrap_or(&Value::from(false))
            .as_bool()
            .unwrap_or(false);

        let original_cmd = cmd.unwrap().as_str().unwrap();

        let shell_param = params.get(&Value::from("shell"));
        let shell = if let Some(shell_param) = shell_param {
            let shell_param = shell_param.as_str().expect("invalid shell parameter");
            let shell = Shell::from_string(shell_param);

            if shell.is_none() {
                error!("Invalid shell parameter, please select a valid one.");
                return Err(super::ExtensionError::Internal);
            }

            shell.unwrap()
        } else {
            Shell::default()
        };

        // Render positional parameters in args
        let cmd = if inject_args {
            shell.get_arg_regex()
            .replace_all(&original_cmd, |caps: &Captures| {
                let position_str = caps.name("pos").unwrap().as_str();
                let position = position_str.parse::<i32>().unwrap_or(-1);
                if position >= 0 && position < args.len() as i32 {
                    args[position as usize].to_owned()
                } else {
                    "".to_owned()
                }
            })
            .to_string()
        } else {
            original_cmd.to_owned()
        };

        let env_variables = super::utils::convert_to_env_variables(&vars);

        let output = shell.execute_cmd(&cmd, &env_variables);

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(output.stdout.as_slice());
                let mut output_str = output_str.into_owned();
                let error_str = String::from_utf8_lossy(output.stderr.as_slice());
                let error_str = error_str.to_string();
                let error_str = error_str.trim();

                // Print stderror if present
                if !error_str.is_empty() {
                    warn!("Shell command reported error: \n{}", error_str);
                }

                // Check if debug flag set, provide additional context when an error occurs.
                let debug_opt = params.get(&Value::from("debug"));
                let with_debug = if let Some(value) = debug_opt {
                    let val = value.as_bool();
                    val.unwrap_or(false)
                } else {
                    false
                };

                if with_debug {
                    info!(
                        "debug for shell cmd '{}', exit_status '{}', stdout '{}', stderr '{}'",
                        original_cmd, output.status, output_str, error_str
                    );
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

                Ok(Some(ExtensionResult::Single(output_str)))
            }
            Err(e) => {
                error!("Could not execute cmd '{}', error: {}", cmd, e);
                Err(super::ExtensionError::Internal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::Extension;

    #[test]
    fn test_shell_not_trimmed() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo \"hello world\""));
        params.insert(Value::from("trim"), Value::from(false));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec![], &HashMap::new()).unwrap();

        assert!(output.is_some());

        if cfg!(target_os = "windows") {
            assert_eq!(
                output.unwrap(),
                ExtensionResult::Single("hello world\r\n".to_owned())
            );
        } else {
            assert_eq!(
                output.unwrap(),
                ExtensionResult::Single("hello world\n".to_owned())
            );
        }
    }

    #[test]
    fn test_shell_basic() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo \"hello world\""));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec![], &HashMap::new()).unwrap();

        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            ExtensionResult::Single("hello world".to_owned())
        );
    }

    #[test]
    fn test_shell_trimmed_2() {
        let mut params = Mapping::new();
        params.insert(
            Value::from("cmd"),
            Value::from("echo \"   hello world     \""),
        );

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec![], &HashMap::new()).unwrap();

        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            ExtensionResult::Single("hello world".to_owned())
        );
    }

    #[test]
    fn test_shell_trimmed_malformed() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo \"hello world\""));
        params.insert(Value::from("trim"), Value::from("error"));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec![], &HashMap::new()).unwrap();

        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            ExtensionResult::Single("hello world".to_owned())
        );
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_shell_pipes() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo hello world | cat"));
        params.insert(Value::from("trim"), Value::from(true));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec![], &HashMap::new()).unwrap();

        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            ExtensionResult::Single("hello world".to_owned())
        );
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_shell_args_unix() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo $0"));
        params.insert(Value::from("inject_args"), Value::from(true));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec!["hello".to_owned()], &HashMap::new()).unwrap();

        assert!(output.is_some());

        assert_eq!(output.unwrap(), ExtensionResult::Single("hello".to_owned()));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_shell_no_default_inject_args_unix() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo 'hey friend' | awk '{ print $2 }'"));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec!["hello".to_owned()], &HashMap::new()).unwrap();

        assert!(output.is_some());

        assert_eq!(output.unwrap(), ExtensionResult::Single("friend".to_owned()));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_shell_args_windows() {
        let mut params = Mapping::new();
        params.insert(Value::from("cmd"), Value::from("echo %0"));
        params.insert(Value::from("inject_args"), Value::from(true));

        let extension = ShellExtension::new();
        let output = extension.calculate(&params, &vec!["hello".to_owned()], &HashMap::new()).unwrap();

        assert!(output.is_some());

        assert_eq!(output.unwrap(), ExtensionResult::Single("hello".to_owned()));
    }

    #[test]
    fn test_shell_vars_single_injection() {
        let mut params = Mapping::new();
        if cfg!(target_os = "windows") {
            params.insert(Value::from("cmd"), Value::from("echo %ESPANSO_VAR1%"));
            params.insert(Value::from("shell"), Value::from("cmd"));
        } else {
            params.insert(Value::from("cmd"), Value::from("echo $ESPANSO_VAR1"));
        }

        let extension = ShellExtension::new();
        let mut vars: HashMap<String, ExtensionResult> = HashMap::new();
        vars.insert(
            "var1".to_owned(),
            ExtensionResult::Single("hello".to_owned()),
        );
        let output = extension.calculate(&params, &vec![], &vars).unwrap();

        assert!(output.is_some());
        assert_eq!(output.unwrap(), ExtensionResult::Single("hello".to_owned()));
    }

    #[test]
    fn test_shell_vars_multiple_injection() {
        let mut params = Mapping::new();
        if cfg!(target_os = "windows") {
            params.insert(Value::from("cmd"), Value::from("echo %ESPANSO_FORM1_NAME%"));
            params.insert(Value::from("shell"), Value::from("cmd"));
        } else {
            params.insert(Value::from("cmd"), Value::from("echo $ESPANSO_FORM1_NAME"));
        }

        let extension = ShellExtension::new();
        let mut vars: HashMap<String, ExtensionResult> = HashMap::new();
        let mut subvars = HashMap::new();
        subvars.insert("name".to_owned(), "John".to_owned());
        vars.insert("form1".to_owned(), ExtensionResult::Multiple(subvars));
        let output = extension.calculate(&params, &vec![], &vars).unwrap();

        assert!(output.is_some());
        assert_eq!(output.unwrap(), ExtensionResult::Single("John".to_owned()));
    }
}
