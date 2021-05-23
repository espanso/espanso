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

use std::collections::HashMap;

use espanso_render::{
  extension::form::{FormProvider, FormProviderResult},
  Params, Value,
};
use log::error;

use crate::gui::{FormField, FormUI};

pub struct FormProviderAdapter<'a> {
  form_ui: &'a dyn FormUI,
}

impl<'a> FormProviderAdapter<'a> {
  pub fn new(form_ui: &'a dyn FormUI) -> Self {
    Self { form_ui }
  }
}

impl<'a> FormProvider for FormProviderAdapter<'a> {
  fn show(&self, layout: &str, fields: &Params, _: &Params) -> FormProviderResult {
    let fields = convert_fields(fields);
    match self.form_ui.show(layout, &fields) {
      Ok(Some(results)) => FormProviderResult::Success(results),
      Ok(None) => FormProviderResult::Aborted,
      Err(err) => FormProviderResult::Error(err),
    }
  }
}

// TODO: test
fn convert_fields(fields: &Params) -> HashMap<String, FormField> {
  let mut out = HashMap::new();
  for (name, field) in fields {
    let mut form_field = None;

    if let Value::Object(params) = field {
      if let Some(Value::String(field_type)) = params.get("type") {
        form_field = match field_type.as_str() {
          "text" => Some(FormField::Text {
            default: params
              .get("default")
              .and_then(|val| val.as_string())
              .cloned(),
            multiline: params
              .get("multiline")
              .and_then(|val| val.as_bool())
              .cloned()
              .unwrap_or(false),
          }),
          "choice" => Some(FormField::Choice {
            default: params
              .get("default")
              .and_then(|val| val.as_string())
              .cloned(),
            values: params
              .get("values")
              .and_then(|val| val.as_array())
              .map(|arr| {
                arr
                  .into_iter()
                  .flat_map(|choice| choice.as_string())
                  .cloned()
                  .collect()
              })
              .unwrap_or_default(),
          }),
          "list" => Some(FormField::List {
            default: params
              .get("default")
              .and_then(|val| val.as_string())
              .cloned(),
            values: params
              .get("values")
              .and_then(|val| val.as_array())
              .map(|arr| {
                arr
                  .into_iter()
                  .flat_map(|choice| choice.as_string())
                  .cloned()
                  .collect()
              })
              .unwrap_or_default(),
          }),
          _ => None,
        }
      }
    }

    if let Some(form_field) = form_field {
      out.insert(name.clone(), form_field);
    } else {
      error!("malformed form field format for '{}'", name);
    }
  }
  out
}
