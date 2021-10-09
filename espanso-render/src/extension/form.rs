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

use crate::renderer::VAR_REGEX;
use log::error;
use std::collections::HashMap;
use thiserror::Error;

use crate::{
  renderer::render_variables, Extension, ExtensionOutput, ExtensionResult, Params, Value,
};

lazy_static! {
  static ref EMPTY_PARAMS: Params = Params::new();
}

pub trait FormProvider {
  fn show(&self, layout: &str, fields: &Params, options: &Params) -> FormProviderResult;
}

pub enum FormProviderResult {
  Success(HashMap<String, String>),
  Aborted,
  Error(anyhow::Error),
}

pub struct FormExtension<'a> {
  provider: &'a dyn FormProvider,
}

#[allow(clippy::new_without_default)]
impl<'a> FormExtension<'a> {
  pub fn new(provider: &'a dyn FormProvider) -> Self {
    Self { provider }
  }
}

impl<'a> Extension for FormExtension<'a> {
  fn name(&self) -> &str {
    "form"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    scope: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    let layout = if let Some(Value::String(layout)) = params.get("layout") {
      layout
    } else {
      return crate::ExtensionResult::Error(FormExtensionError::MissingLayout.into());
    };

    let mut fields = if let Some(Value::Object(fields)) = params.get("fields") {
      fields.clone()
    } else {
      Params::new()
    };

    // Inject scope variables into fields (if needed)
    inject_scope(&mut fields, scope);

    match self.provider.show(layout, &fields, &EMPTY_PARAMS) {
      FormProviderResult::Success(values) => {
        ExtensionResult::Success(ExtensionOutput::Multiple(values))
      }
      FormProviderResult::Aborted => ExtensionResult::Aborted,
      FormProviderResult::Error(error) => ExtensionResult::Error(error),
    }
  }
}

// TODO: test
fn inject_scope(fields: &mut HashMap<String, Value>, scope: &HashMap<&str, ExtensionOutput>) {
  for value in fields.values_mut() {
    if let Value::Object(field_options) = value {
      if let Some(Value::String(default_value)) = field_options.get_mut("default") {
        if VAR_REGEX.is_match(default_value) {
          match render_variables(default_value, scope) {
            Ok(rendered) => *default_value = rendered,
            Err(err) => error!(
              "error while injecting variable in form default value: {}",
              err
            ),
          }
        }
      }
    }
  }
}

#[derive(Error, Debug)]
pub enum FormExtensionError {
  #[error("missing layout parameter")]
  MissingLayout,
}
