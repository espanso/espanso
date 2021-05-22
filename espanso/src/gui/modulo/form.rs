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

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::{collections::HashMap};

use crate::gui::{FormField, FormUI};

use super::manager::ModuloManager;

pub struct ModuloFormUI<'a> {
  manager: &'a ModuloManager,
}

impl<'a> ModuloFormUI<'a> {
  pub fn new(manager: &'a ModuloManager) -> Self {
    Self {
      manager,
    }
  }
}

impl<'a> FormUI for ModuloFormUI<'a> {
  fn show(
    &self,
    layout: &str,
    fields: &HashMap<String, FormField>,
  ) -> anyhow::Result<Option<HashMap<String, String>>> {
    let modulo_form_config = ModuloFormConfig {
      title: "espanso",
      layout,
      fields: convert_fields_into_object(fields),
    };

    let json_config = serde_json::to_string(&modulo_form_config)?;
    let output = self
      .manager
      .invoke(&["form", "-j", "-i", "-"], &json_config)?;
    let json: Result<HashMap<String, String>, _> = serde_json::from_str(&output);
    match json {
      Ok(json) => {
        if json.is_empty() {
          return Ok(None);
        } else {
          return Ok(Some(json));
        }
      }
      Err(error) => {
        return Err(error.into());
      }
    }
  }
}

#[derive(Debug, Serialize)]
struct ModuloFormConfig<'a> {
  title: &'a str,
  layout: &'a str,
  fields: Map<String, Value>,
}

// TODO: test
fn convert_fields_into_object(fields: &HashMap<String, FormField>) -> Map<String, Value> {
  let mut obj = Map::new();
  for (name, field) in fields {
    let value = match field {
      FormField::Text { default, multiline } => json!({
        "type": "text",
        "default": default,
        "multiline": multiline,
      }),
      FormField::Choice { default, values } => json!({
        "type": "choice",
        "default": default,
        "values": values,
      }),
      FormField::List { default, values } => json!({
        "type": "list",
        "default": default,
        "values": values,
      }),
    };
    obj.insert(name.clone(), value);
  }
  obj
}
