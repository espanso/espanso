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

use anyhow::Result;
use log::error;
use thiserror::Error;

use crate::{Extension, ExtensionOutput, ExtensionResult, Params, Value};

pub trait ChoiceSelector {
  fn show(&self, choices: &[Choice]) -> ChoiceSelectorResult;
}

#[derive(Debug, Clone)]
pub struct Choice<'a> {
  pub label: &'a str,
  pub id: &'a str,
}

pub enum ChoiceSelectorResult {
  Success(String),
  Aborted,
  Error(anyhow::Error),
}

pub struct ChoiceExtension<'a> {
  selector: &'a dyn ChoiceSelector,
}

#[allow(clippy::new_without_default)]
impl<'a> ChoiceExtension<'a> {
  pub fn new(selector: &'a dyn ChoiceSelector) -> Self {
    Self { selector }
  }
}

impl<'a> Extension for ChoiceExtension<'a> {
  fn name(&self) -> &str {
    "choice"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    _: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    let choices: Vec<Choice> = if let Some(Value::String(values)) = params.get("values") {
      values
        .lines()
        .filter_map(|line| {
          let trimmed_line = line.trim();
          if trimmed_line.is_empty() {
            None
          } else {
            Some(trimmed_line)
          }
        })
        .map(|line| Choice {
          label: line,
          id: line,
        })
        .collect()
    } else if let Some(Value::Array(values)) = params.get("values") {
      let choices: Result<Vec<Choice>> = values
        .iter()
        .map(|value| match value {
          Value::String(string) => Ok(Choice {
            id: string,
            label: string,
          }),
          Value::Object(fields) => Ok(Choice {
            id: fields
              .get("id")
              .and_then(|val| val.as_string())
              .ok_or(ChoiceError::InvalidObjectValue)?,
            label: fields
              .get("label")
              .and_then(|val| val.as_string())
              .ok_or(ChoiceError::InvalidObjectValue)?,
          }),
          _ => Err(ChoiceError::InvalidValueType.into()),
        })
        .collect();

      match choices {
        Ok(choices) => choices,
        Err(err) => {
          return crate::ExtensionResult::Error(err);
        }
      }
    } else {
      return crate::ExtensionResult::Error(ChoiceError::MissingValues.into());
    };

    match self.selector.show(&choices) {
      ChoiceSelectorResult::Success(choice_id) => {
        ExtensionResult::Success(ExtensionOutput::Single(choice_id))
      }
      ChoiceSelectorResult::Aborted => ExtensionResult::Aborted,
      ChoiceSelectorResult::Error(error) => ExtensionResult::Error(error),
    }
  }
}

#[derive(Error, Debug)]
pub enum ChoiceError {
  #[error("missing values parameter")]
  MissingValues,

  #[error("values contain object items, but they are missing either the 'id' or 'label' fields")]
  InvalidObjectValue,

  #[error("values contain an invalid item type. items can only be strings or objects")]
  InvalidValueType,
}
