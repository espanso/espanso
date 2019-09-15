use serde_yaml::{Mapping, Value};
use std::process::Command;
use log::{warn, error};

pub struct ScriptExtension {}

impl ScriptExtension {
    pub fn new() -> ScriptExtension {
        ScriptExtension{}
    }
}

impl super::Extension for ScriptExtension {
    fn name(&self) -> String {
        String::from("script")
    }

    fn calculate(&self, params: &Mapping) -> Option<String> {
        let args = params.get(&Value::from("args"));
        if args.is_none() {
            warn!("No 'args' parameter specified for script variable");
            return None
        }
        let args = args.unwrap().as_sequence();
        if let Some(args) = args {
            let str_args = args.iter().map(|arg| {
               arg.as_str().unwrap_or_default().to_string()
            }).collect::<Vec<String>>();

            let output = if str_args.len() > 1 {
                Command::new(&str_args[0])
                    .args(&str_args[1..])
                    .output()
            }else{
                Command::new(&str_args[0])
                    .output()
            };

            match output {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(output.stdout.as_slice());

                    return Some(output_str.into_owned())
                },
                Err(e) => {
                    error!("Could not execute script '{:?}', error: {}", args, e);
                    return None
                },
            }
        }

        error!("Could not execute script with args '{:?}'", args);
        None
    }
}