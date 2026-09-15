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

use crate::{Extension, ExtensionOutput, ExtensionResult, Number, Params, Value};
use thiserror::Error;

pub struct EchoExtension {
    alias: String,
}

impl EchoExtension {
    pub fn new() -> Self {
        Self {
            alias: "echo".to_string(),
        }
    }
}

impl Default for EchoExtension {
    fn default() -> Self {
        Self::new()
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
        // YAML scalars arrive unquoted as numbers and booleans, so coerce them
        // rather than making users quote every value
        let echo = match params.get("echo") {
            Some(Value::String(value)) => Some(value.clone()),
            Some(Value::Number(Number::Integer(value))) => Some(value.to_string()),
            Some(Value::Number(Number::Float(value))) => Some(value.to_string()),
            Some(Value::Bool(value)) => Some(value.to_string()),
            _ => None,
        };

        match echo {
            Some(echo) => ExtensionResult::Success(ExtensionOutput::Single(echo)),
            None => ExtensionResult::Error(EchoExtensionError::MissingEchoParameter.into()),
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
    use std::collections::HashMap;

    use super::*;

    fn echo(value: Value) -> ExtensionResult {
        let extension = EchoExtension::new();
        let param = vec![("echo".to_string(), value)]
            .into_iter()
            .collect::<Params>();
        extension.calculate(&crate::Context::default(), &HashMap::default(), &param)
    }

    #[test]
    fn echo_works_correctly() {
        let extension = EchoExtension::new();

        let param = vec![("echo".to_string(), Value::String("test".to_string()))]
            .into_iter()
            .collect::<Params>();
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("test".to_string())
        );
    }

    #[test]
    fn echo_coerces_scalars() {
        for (value, expected) in [
            (Value::Number(Number::Integer(12345)), "12345"),
            (Value::Number(Number::Integer(-1)), "-1"),
            (Value::Number(Number::Float(1.0)), "1"),
            (Value::Number(Number::Float(1.5)), "1.5"),
            (Value::Bool(true), "true"),
            (Value::Bool(false), "false"),
        ] {
            assert_eq!(
                echo(value).into_success().unwrap(),
                ExtensionOutput::Single(expected.to_string())
            );
        }
    }

    #[test]
    fn echo_rejects_non_scalars() {
        for value in [
            Value::Null,
            Value::Array(vec![Value::Bool(true)]),
            Value::Object(HashMap::new()),
        ] {
            assert!(matches!(echo(value), ExtensionResult::Error(_)));
        }
    }

    #[test]
    fn missing_echo_parameter() {
        let extension = EchoExtension::new();

        let param = Params::new();
        assert!(matches!(
            extension.calculate(&crate::Context::default(), &HashMap::default(), &param),
            ExtensionResult::Error(_)
        ));
    }
}
