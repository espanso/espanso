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