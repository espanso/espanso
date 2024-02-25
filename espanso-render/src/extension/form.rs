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

use lazy_static::lazy_static;
use log::error;
use std::collections::HashMap;
use thiserror::Error;

use crate::{Extension, ExtensionOutput, ExtensionResult, Params, Value};

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
    _: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    let Some(Value::String(layout)) = params.get("layout") else {
      return crate::ExtensionResult::Error(FormExtensionError::MissingLayout.into());
    };

    let fields = if let Some(Value::Object(fields)) = params.get("fields") {
      fields.clone()
    } else {
      Params::new()
    };

    match self.provider.show(layout, &fields, &EMPTY_PARAMS) {
      FormProviderResult::Success(values) => {
        ExtensionResult::Success(ExtensionOutput::Multiple(values))
      }
      FormProviderResult::Aborted => ExtensionResult::Aborted,
      FormProviderResult::Error(error) => ExtensionResult::Error(error),
    }
  }
}

#[derive(Error, Debug)]
pub enum FormExtensionError {
  #[error("missing layout parameter")]
  MissingLayout,
}
