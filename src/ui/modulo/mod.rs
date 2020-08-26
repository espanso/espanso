use crate::config::Configs;
use log::{error, info};
use std::io::{Error, Write};
use std::process::{Child, Command, Output};

pub struct ModuloManager {
    modulo_path: Option<String>,
}

impl ModuloManager {
    pub fn new(config: &Configs) -> Self {
        let mut modulo_path: Option<String> = None;
        // Check if the `MODULO_PATH` env variable is configured
        if let Some(_modulo_path) = std::env::var_os("MODULO_PATH") {
            modulo_path = Some(_modulo_path.to_string_lossy().to_string())
        } else if let Some(ref _modulo_path) = config.modulo_path {
            // Check the configs
            modulo_path = Some(_modulo_path.to_owned());
        } else {
            // Check in the same directory of espanso
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let possible_path = parent.join("modulo");
                    let possible_path = possible_path.to_string_lossy().to_string();

                    if let Ok(output) = Command::new(&possible_path).arg("--version").output() {
                        if output.status.success() {
                            modulo_path = Some(possible_path);
                        }
                    }
                }
            }

            // Otherwise check if present in the PATH
            if modulo_path.is_none() {
                if let Ok(output) = Command::new("modulo").arg("--version").output() {
                    if output.status.success() {
                        modulo_path = Some("modulo".to_owned());
                    }
                }
            }
        }

        if let Some(ref modulo_path) = modulo_path {
            info!("Using modulo at {:?}", modulo_path);
        }

        Self { modulo_path }
    }

    pub fn is_valid(&self) -> bool {
        self.modulo_path.is_some()
    }

    pub fn get_version(&self) -> Option<String> {
        if let Some(ref modulo_path) = self.modulo_path {
            if let Ok(output) = Command::new(modulo_path).arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                return Some(version.to_string());
            }
        }

        None
    }

    pub fn invoke(&self, args: &[&str], body: &str) -> Option<String> {
        if self.modulo_path.is_none() {
            error!("Attempt to invoke modulo even though it's not configured");
            return None;
        }

        if let Some(ref modulo_path) = self.modulo_path {
            let mut command = Command::new(modulo_path);
            command
                .args(args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            crate::utils::set_command_flags(&mut command);

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

                                        return Some(output.to_string());
                                    }
                                    Err(error) => {
                                        error!("error while getting output from modulo: {}", error);
                                    }
                                }
                            }
                            Err(error) => {
                                error!("error while sending body to modulo: {}", error);
                            }
                        }
                    } else {
                        error!("unable to open stdin to modulo");
                    }
                }
                Err(error) => {
                    error!("error reported when invoking modulo: {}", error);
                }
            }
        }

        None
    }
}
