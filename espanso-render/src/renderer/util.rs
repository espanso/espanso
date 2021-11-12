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

use crate::{renderer::RendererError, ExtensionOutput, Params, Scope, Value};
use anyhow::Result;
use log::error;
use regex::Captures;

use super::VAR_REGEX;
use std::collections::HashSet;

pub(crate) fn get_body_variable_names(body: &str) -> HashSet<&str> {
  let mut variables = HashSet::new();
  for caps in VAR_REGEX.captures_iter(body) {
    let var_name = caps.name("name").unwrap().as_str();
    variables.insert(var_name);
  }
  variables
}

pub(crate) fn get_params_variable_names(params: &Params) -> HashSet<&str> {
  let mut names = HashSet::new();

  for (_, value) in params.iter() {
    let local_names = get_value_variable_names_recursively(value);
    names.extend(local_names);
  }

  names
}

fn get_value_variable_names_recursively(value: &Value) -> HashSet<&str> {
  match value {
    Value::String(s_value) => get_body_variable_names(s_value),
    Value::Array(values) => {
      let mut local_names: HashSet<&str> = HashSet::new();
      for value in values {
        local_names.extend(get_value_variable_names_recursively(value));
      }
      local_names
    }
    Value::Object(fields) => {
      let mut local_names: HashSet<&str> = HashSet::new();
      for value in fields.values() {
        local_names.extend(get_value_variable_names_recursively(value));
      }
      local_names
    }
    _ => HashSet::new(),
  }
}

pub(crate) fn render_variables(body: &str, scope: &Scope) -> Result<String> {
  let mut replacing_error = None;
  let output = VAR_REGEX
    .replace_all(body, |caps: &Captures| {
      let var_name = caps.name("name").unwrap().as_str();
      let var_subname = caps.name("subname");
      match scope.get(var_name) {
        Some(output) => match output {
          ExtensionOutput::Single(output) => output,
          ExtensionOutput::Multiple(results) => match var_subname {
            Some(var_subname) => {
              let var_subname = var_subname.as_str();
              results.get(var_subname).map_or("", |value| &*value)
            }
            None => {
              error!(
                "nested name missing from multi-value variable: {}",
                var_name
              );
              replacing_error = Some(RendererError::MissingVariable(format!(
                "nested name missing from multi-value variable: {}",
                var_name
              )));
              ""
            }
          },
        },
        None => {
          replacing_error = Some(RendererError::MissingVariable(format!(
            "variable '{}' is missing",
            var_name
          )));
          ""
        }
      }
    })
    .to_string();

  if let Some(error) = replacing_error {
    return Err(error.into());
  }

  let unescaped_output = unescape_variable_inections(&output);
  Ok(unescaped_output)
}

pub(crate) fn unescape_variable_inections(body: &str) -> String {
  body.replace("\\{\\{", "{{").replace("\\}\\}", "}}")
}

pub(crate) fn inject_variables_into_params(params: &Params, scope: &Scope) -> Result<Params> {
  let mut params = params.clone();

  for (_, value) in params.iter_mut() {
    inject_variables_into_value(value, scope)?;
  }

  Ok(params)
}

fn inject_variables_into_value(value: &mut Value, scope: &Scope) -> Result<()> {
  match value {
    Value::String(s_value) => {
      let new_value = render_variables(s_value, scope)?;

      if &new_value != s_value {
        s_value.clear();
        s_value.push_str(&new_value);
      }
    }
    Value::Array(values) => {
      for value in values {
        inject_variables_into_value(value, scope)?;
      }
    }
    Value::Object(fields) => {
      for value in fields.values_mut() {
        inject_variables_into_value(value, scope)?;
      }
    }
    _ => {}
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{collections::HashMap, iter::FromIterator};

  #[test]
  fn get_body_variable_names_no_vars() {
    assert_eq!(
      get_body_variable_names("no variables"),
      HashSet::from_iter(vec![]),
    );
  }

  #[test]
  fn get_body_variable_names_multiple_vars() {
    assert_eq!(
      get_body_variable_names("hello {{world}} name {{greet}}"),
      HashSet::from_iter(vec!["world", "greet"]),
    );
  }

  #[test]
  fn test_inject_variables_into_params() {
    let mut params = Params::new();
    params.insert(
      "field1".to_string(),
      Value::String("this contains {{first}}".to_string()),
    );
    params.insert("field2".to_string(), Value::Bool(true));
    params.insert(
      "field3".to_string(),
      Value::Array(vec![Value::String("this contains {{first}}".to_string())]),
    );

    let mut nested = HashMap::new();
    nested.insert(
      "subfield1".to_string(),
      Value::String("also contains {{first}}".to_string()),
    );
    params.insert("field4".to_string(), Value::Object(nested));

    let mut scope = Scope::new();
    scope.insert("first", ExtensionOutput::Single("one".to_string()));

    let result = inject_variables_into_params(&params, &scope).unwrap();

    assert_eq!(result.len(), 4);
    assert_eq!(
      result.get("field1").unwrap(),
      &Value::String("this contains one".to_string())
    );
    assert_eq!(result.get("field2").unwrap(), &Value::Bool(true));
    assert_eq!(
      result.get("field3").unwrap(),
      &Value::Array(vec![Value::String("this contains one".to_string())])
    );
    assert!(
      matches!(result.get("field4").unwrap(), Value::Object(fields) if fields.get("subfield1").unwrap() == &Value::String("also contains one".to_string()))
    );
  }
}
