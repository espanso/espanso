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

use crate::{Extension, ExtensionOutput, ExtensionResult, Params, Value};
use rand::seq::SliceRandom;
use thiserror::Error;

pub struct RandomExtension {}

#[allow(clippy::new_without_default)]
impl RandomExtension {
  pub fn new() -> Self {
    Self {}
  }
}

impl Extension for RandomExtension {
  fn name(&self) -> &str {
    "random"
  }

  fn calculate(
    &self,
    _: &crate::Context,
    _: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    if let Some(Value::Array(choices)) = params.get("choices") {
      let choices: Vec<String> = choices
        .iter()
        .filter_map(|arg| arg.as_string())
        .cloned()
        .collect();

      if let Some(choice) = choices.choose(&mut rand::thread_rng()) {
        ExtensionResult::Success(ExtensionOutput::Single(choice.clone()))
      } else {
        ExtensionResult::Error(RandomExtensionError::SelectionError.into())
      }
    } else {
      ExtensionResult::Error(RandomExtensionError::MissingChoicesParameter.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum RandomExtensionError {
  #[error("missing 'choices' parameter")]
  MissingChoicesParameter,

  #[error("could not select a choice randomly")]
  SelectionError,
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;

  #[test]
  fn random_works_correctly() {
    let extension = RandomExtension::new();

    let param = vec![(
      "choices".to_string(),
      Value::Array(vec![
        Value::String("first".to_string()),
        Value::String("second".to_string()),
        Value::String("third".to_string()),
      ]),
    )]
    .into_iter()
    .collect::<Params>();
    assert!(matches!(
      extension
        .calculate(&crate::Context::default(), &HashMap::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single(result) if ["first", "second", "third"].contains(&result.as_str())
    ));
  }

  #[test]
  fn missing_echo_parameter() {
    let extension = RandomExtension::new();

    let param = Params::new();
    assert!(matches!(
      extension.calculate(&crate::Context::default(), &HashMap::default(), &param),
      ExtensionResult::Error(_)
    ));
  }
}
