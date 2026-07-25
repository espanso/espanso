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

use std::convert::TryInto;

use anyhow::Result;
use yaml_serde::Mapping;
use thiserror::Error;

use crate::matches::{Number, Params, Value};

pub fn convert_params(m: Mapping) -> Result<Params> {
    let mut params = Params::new();

    for (key, value) in m {
        let key = key.as_str().ok_or(ConversionError::InvalidKeyFormat)?;
        let value = convert_value(value)?;
        params.insert(key.to_owned(), value);
    }

    Ok(params)
}

fn convert_value(value: yaml_serde::Value) -> Result<Value> {
    Ok(match value {
        yaml_serde::Value::Null => Value::Null,
        yaml_serde::Value::Bool(val) => Value::Bool(val),
        yaml_serde::Value::Number(n) => {
            if n.is_i64() {
                Value::Number(Number::Integer(
                    n.as_i64().ok_or(ConversionError::InvalidNumberFormat)?,
                ))
            } else if n.is_u64() {
                Value::Number(Number::Integer(
                    n.as_u64()
                        .ok_or(ConversionError::InvalidNumberFormat)?
                        .try_into()?,
                ))
            } else if n.is_f64() {
                Value::Number(Number::Float(
                    n.as_f64()
                        .ok_or(ConversionError::InvalidNumberFormat)?
                        .into(),
                ))
            } else {
                return Err(ConversionError::InvalidNumberFormat.into());
            }
        }
        yaml_serde::Value::String(s) => Value::String(s),
        yaml_serde::Value::Sequence(arr) => Value::Array(
            arr.into_iter()
                .map(convert_value)
                .collect::<Result<Vec<Value>>>()?,
        ),
        yaml_serde::Value::Mapping(m) => Value::Object(convert_params(m)?),
        yaml_serde::Value::Tagged(tagged) => convert_value(tagged.value)?,
    })
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("invalid key format")]
    InvalidKeyFormat,

    #[error("invalid number format")]
    InvalidNumberFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_value_null() {
        assert_eq!(
            convert_value(yaml_serde::Value::Null).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn convert_value_bool() {
        assert_eq!(
            convert_value(yaml_serde::Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            convert_value(yaml_serde::Value::Bool(false)).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn convert_value_number() {
        assert_eq!(
            convert_value(yaml_serde::Value::Number(0.into())).unwrap(),
            Value::Number(Number::Integer(0))
        );
        assert_eq!(
            convert_value(yaml_serde::Value::Number((-100).into())).unwrap(),
            Value::Number(Number::Integer(-100))
        );
        assert_eq!(
            convert_value(yaml_serde::Value::Number(1.5.into())).unwrap(),
            Value::Number(Number::Float(1.5.into()))
        );
    }
    #[test]
    fn convert_value_string() {
        assert_eq!(
            convert_value(yaml_serde::Value::String("hello".to_string())).unwrap(),
            Value::String("hello".to_string())
        );
    }
    #[test]
    fn convert_value_array() {
        assert_eq!(
            convert_value(yaml_serde::Value::Sequence(vec![
                yaml_serde::Value::Bool(true),
                yaml_serde::Value::Null,
            ]))
            .unwrap(),
            Value::Array(vec![Value::Bool(true), Value::Null,])
        );
    }

    #[test]
    fn convert_value_params() {
        let mut mapping = yaml_serde::Mapping::new();
        mapping.insert(
            yaml_serde::Value::String("test".to_string()),
            yaml_serde::Value::Null,
        );

        let mut expected = Params::new();
        expected.insert("test".to_string(), Value::Null);
        assert_eq!(
            convert_value(yaml_serde::Value::Mapping(mapping)).unwrap(),
            Value::Object(expected)
        );
    }

    #[test]
    fn convert_params_works_correctly() {
        let mut mapping = yaml_serde::Mapping::new();
        mapping.insert(
            yaml_serde::Value::String("test".to_string()),
            yaml_serde::Value::Null,
        );

        let mut expected = Params::new();
        expected.insert("test".to_string(), Value::Null);
        assert_eq!(convert_params(mapping).unwrap(), expected);
    }

    #[test]
    fn convert_params_invalid_key_type() {
        let mut mapping = yaml_serde::Mapping::new();
        mapping.insert(yaml_serde::Value::Null, yaml_serde::Value::Null);

        assert!(convert_params(mapping).is_err());
    }
}
