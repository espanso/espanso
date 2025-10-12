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
use serde_norway::Mapping;
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

fn convert_value(value: serde_norway::Value) -> Result<Value> {
    Ok(match value {
        serde_norway::Value::Null => Value::Null,
        serde_norway::Value::Bool(val) => Value::Bool(val),
        serde_norway::Value::Number(n) => {
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
        serde_norway::Value::String(s) => Value::String(s),
        serde_norway::Value::Sequence(arr) => Value::Array(
            arr.into_iter()
                .map(convert_value)
                .collect::<Result<Vec<Value>>>()?,
        ),
        serde_norway::Value::Mapping(m) => Value::Object(convert_params(m)?),
        serde_norway::Value::Tagged(tagged) => convert_value(tagged.value)?,
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
        assert_eq!(convert_value(serde_norway::Value::Null).unwrap(), Value::Null);
    }

    #[test]
    fn convert_value_bool() {
        assert_eq!(
            convert_value(serde_norway::Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            convert_value(serde_norway::Value::Bool(false)).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn convert_value_number() {
        assert_eq!(
            convert_value(serde_norway::Value::Number(0.into())).unwrap(),
            Value::Number(Number::Integer(0))
        );
        assert_eq!(
            convert_value(serde_norway::Value::Number((-100).into())).unwrap(),
            Value::Number(Number::Integer(-100))
        );
        assert_eq!(
            convert_value(serde_norway::Value::Number(1.5.into())).unwrap(),
            Value::Number(Number::Float(1.5.into()))
        );
    }
    #[test]
    fn convert_value_string() {
        assert_eq!(
            convert_value(serde_norway::Value::String("hello".to_string())).unwrap(),
            Value::String("hello".to_string())
        );
    }
    #[test]
    fn convert_value_array() {
        assert_eq!(
            convert_value(serde_norway::Value::Sequence(vec![
                serde_norway::Value::Bool(true),
                serde_norway::Value::Null,
            ]))
            .unwrap(),
            Value::Array(vec![Value::Bool(true), Value::Null,])
        );
    }

    #[test]
    fn convert_value_params() {
        let mut mapping = serde_norway::Mapping::new();
        mapping.insert(
            serde_norway::Value::String("test".to_string()),
            serde_norway::Value::Null,
        );

        let mut expected = Params::new();
        expected.insert("test".to_string(), Value::Null);
        assert_eq!(
            convert_value(serde_norway::Value::Mapping(mapping)).unwrap(),
            Value::Object(expected)
        );
    }

    #[test]
    fn convert_params_works_correctly() {
        let mut mapping = serde_norway::Mapping::new();
        mapping.insert(
            serde_norway::Value::String("test".to_string()),
            serde_norway::Value::Null,
        );

        let mut expected = Params::new();
        expected.insert("test".to_string(), Value::Null);
        assert_eq!(convert_params(mapping).unwrap(), expected);
    }

    #[test]
    fn convert_params_invalid_key_type() {
        let mut mapping = serde_norway::Mapping::new();
        mapping.insert(serde_norway::Value::Null, serde_norway::Value::Null);

        assert!(convert_params(mapping).is_err());
    }
}
