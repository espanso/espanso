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

use std::{collections::HashMap, path::PathBuf};

use super::ModuloManager;
use anyhow::Result;
use espanso_render::extension::form::{FormProvider, FormProviderResult};
use log::{error};
use serde::Serialize;
use serde_json::{Map, Value};

pub struct ModuloFormProviderAdapter<'a> {
  manager: &'a ModuloManager,
  icon_path: Option<String>,
}

impl<'a> ModuloFormProviderAdapter<'a> {
  pub fn new(manager: &'a ModuloManager, icon_path: Option<PathBuf>) -> Self {
    Self {
      manager,
      icon_path: icon_path.map(|path| path.to_string_lossy().to_string()),
    }
  }
}

impl<'a> FormProvider for ModuloFormProviderAdapter<'a> {
  fn show(
    &self,
    layout: &str,
    fields: &espanso_render::Params,
    _: &espanso_render::Params,
  ) -> FormProviderResult {
    let modulo_form_config = ModuloFormConfig {
      icon: self.icon_path.as_deref(),
      title: "espanso",
      layout,
      fields: convert_params_into_object(fields),
    };

    match serde_json::to_string(&modulo_form_config) {
      Ok(json_config) => {
        match self
          .manager
          .invoke(&["form", "-j", "-i", "-"], &json_config)
        {
          Ok(output) => {
            let json: Result<HashMap<String, String>, _> = serde_json::from_str(&output);
            match json {
              Ok(json) => {
                if json.is_empty() {
                  return FormProviderResult::Aborted;
                } else {
                  return FormProviderResult::Success(json);
                }
              }
              Err(error) => {
                return FormProviderResult::Error(error.into());
              }
            }
          }
          Err(err) => {
            return FormProviderResult::Error(err.into());
          }
        }
      }
      Err(err) => {
        return FormProviderResult::Error(err.into());
      }
    }
  }
}

#[derive(Debug, Serialize)]
struct ModuloFormConfig<'a> {
  icon: Option<&'a str>,
  title: &'a str,
  layout: &'a str,
  fields: Map<String, Value>,
}

// TODO: test
fn convert_params_into_object(params: &espanso_render::Params) -> Map<String, Value> {
  let mut obj = Map::new();
  for (field, value) in params {
    obj.insert(field.clone(), convert_value(value));
  }
  obj
}

// TODO: test
fn convert_value(value: &espanso_render::Value) -> Value {
  match value {
    espanso_render::Value::Null => Value::Null,
    espanso_render::Value::Bool(value) => Value::Bool(*value),
    espanso_render::Value::Number(num) => match num {
      espanso_render::Number::Integer(val) => Value::Number((*val).into()),
      espanso_render::Number::Float(val) => {
        Value::Number(serde_json::Number::from_f64(*val).unwrap_or_else(|| {
          error!("unable to convert float value to json");
          0.into()
        }))
      }
    },
    espanso_render::Value::String(value) => Value::String(value.clone()),
    espanso_render::Value::Array(arr) => Value::Array(arr.into_iter().map(convert_value).collect()),
    espanso_render::Value::Object(obj) => Value::Object(convert_params_into_object(obj)),
  }
}
