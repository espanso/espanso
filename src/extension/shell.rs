use serde_yaml::{Mapping, Value};
use std::process::Command;
use log::{warn, error};

pub struct ShellExtension {}

impl ShellExtension {
    pub fn new() -> ShellExtension {
        ShellExtension{}
    }
}

impl super::Extension for ShellExtension {
    fn name(&self) -> String {
        String::from("shell")
    }

    fn calculate(&self, params: &Mapping) -> Option<String> {
        let cmd = params.get(&Value::from("cmd"));
        if cmd.is_none() {
            warn!("No 'cmd' parameter specified for shell variable");
            return None
        }
        let cmd = cmd.unwrap().as_str().unwrap();

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", cmd])
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
        };

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(output.stdout.as_slice());

                Some(output_str.into_owned())
            },
            Err(e) => {
                error!("Could not execute cmd '{}', error: {}", cmd, e);
                None
            },
        }
    }
}