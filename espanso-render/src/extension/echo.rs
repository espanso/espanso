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
use thiserror::Error;

pub struct EchoExtension {
  alias: String
}

#[allow(clippy::new_without_default)]
impl EchoExtension {
  pub fn new() -> Self {
    Self {
      alias: "echo".to_string(),
    }
  }

  pub fn new_with_alias(alias: &str) -> Self {
    Self {
      alias: alias.to_string(),
    }
  }
}

impl Extension for EchoExtension {
  fn name(&self) -> &str {
    self.alias.as_str()
  }

  fn calculate(
    &self,
    _: &crate::Context,
    _: &crate::Scope,
    params: &Params,
  ) -> crate::ExtensionResult {
    if let Some(Value::String(echo)) = params.get("echo") {
      ExtensionResult::Success(ExtensionOutput::Single(echo.clone()))
    } else {
      ExtensionResult::Error(EchoExtensionError::MissingEchoParameter.into())
    }
  }
}

#[derive(Error, Debug)]
pub enum EchoExtensionError {
  #[error("missing 'echo' parameter")]
  MissingEchoParameter,
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::iter::FromIterator;

  #[test]
  fn echo_works_correctly() {
    let extension = EchoExtension::new();

    let param =
      Params::from_iter(vec![("echo".to_string(), Value::String("test".to_string()))].into_iter());
    assert_eq!(
      extension
        .calculate(&Default::default(), &Default::default(), &param)
        .into_success()
        .unwrap(),
      ExtensionOutput::Single("test".to_string())
    );
  }

  #[test]
  fn missing_echo_parameter() {
    let extension = EchoExtension::new();

    let param = Params::new();
    assert!(matches!(extension.calculate(&Default::default(), &Default::default(), &param), ExtensionResult::Error(_)));
  }

  #[test]
  fn alias() {
    let extension_with_alias = EchoExtension::new_with_alias("dummy");
    let extension = EchoExtension::new();

    assert_eq!(extension.name(), "echo");
    assert_eq!(extension_with_alias.name(), "dummy");
  }
}
