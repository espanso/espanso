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

use crate::{ExtensionOutput, Scope};
use std::{collections::HashMap, process::Command};

pub fn convert_to_env_variables(scope: &Scope) -> HashMap<String, String> {
  let mut output = HashMap::new();

  for (key, result) in scope {
    match result {
      ExtensionOutput::Single(value) => {
        let name = format!("ESPANSO_{}", key.to_uppercase());
        output.insert(name, value.clone());
      }
      ExtensionOutput::Multiple(values) => {
        for (sub_key, sub_value) in values {
          let name = format!("ESPANSO_{}_{}", key.to_uppercase(), sub_key.to_uppercase());
          output.insert(name, sub_value.clone());
        }
      }
    }
  }

  output
}

#[cfg(target_os = "windows")]
pub fn set_command_flags(command: &mut Command) {
  use std::os::windows::process::CommandExt;
  // Avoid showing the shell window
  // See: https://github.com/espanso/espanso/issues/249
  command.creation_flags(0x0800_0000);
}

#[cfg(not(target_os = "windows"))]
pub fn set_command_flags(_: &mut Command) {
  // NOOP on Linux and macOS
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert_to_env_variables() {
    let mut vars: Scope = HashMap::new();
    let mut subvars = HashMap::new();
    subvars.insert("name".to_owned(), "John".to_owned());
    subvars.insert("lastname".to_owned(), "Snow".to_owned());
    vars.insert("form1", ExtensionOutput::Multiple(subvars));
    vars.insert("var1", ExtensionOutput::Single("test".to_owned()));

    let output = convert_to_env_variables(&vars);
    assert_eq!(output.get("ESPANSO_FORM1_NAME").unwrap(), "John");
    assert_eq!(output.get("ESPANSO_FORM1_LASTNAME").unwrap(), "Snow");
    assert_eq!(output.get("ESPANSO_VAR1").unwrap(), "test");
  }
}
