/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2022 Federico Terzi
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
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

pub struct RegexTransformExtension {
    alias: String,
}

impl RegexTransformExtension {
    pub fn new() -> Self {
        Self {
            alias: "regex_transform".to_string(),
        }
    }
}

impl Default for RegexTransformExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for RegexTransformExtension {
    fn name(&self) -> &str {
        self.alias.as_str()
    }

    fn calculate(
        &self,
        _: &crate::Context,
        _: &crate::Scope,
        params: &Params,
    ) -> crate::ExtensionResult {
        let source = if let Some(Value::String(source)) = params.get("source") {
            source
        } else {
            return ExtensionResult::Error(
                RegexTransformExtensionError::MissingSourceParameter.into(),
            );
        };

        let find = if let Some(Value::String(find)) = params.get("find") {
            find
        } else {
            return ExtensionResult::Error(
                RegexTransformExtensionError::MissingFindParameter.into(),
            );
        };

        let replace = if let Some(Value::String(replace)) = params.get("replace") {
            replace
        } else {
            return ExtensionResult::Error(
                RegexTransformExtensionError::MissingReplaceParameter.into(),
            );
        };

        let modifiers = if let Some(Value::Object(modifiers_map)) = params.get("modifiers") {
            let mut parsed_modifiers = HashMap::new();
            for (key, val) in modifiers_map {
                if let Ok(group_num) = key.parse::<usize>() {
                    if let Value::String(modifier_str) = val {
                        parsed_modifiers.insert(group_num, modifier_str.to_lowercase());
                    }
                }
            }
            parsed_modifiers
        } else {
            HashMap::new()
        };

        let regex = match Regex::new(find) {
            Ok(r) => r,
            Err(e) => {
                return ExtensionResult::Error(
                    RegexTransformExtensionError::InvalidRegex(e.to_string()).into(),
                )
            }
        };

        let result = regex.replace_all(source, |caps: &regex::Captures| {
            // Because replace_all closure doesn't have a direct way to build the replacement string
            // with mapped capture groups while easily substituting them into the `replace` template,
            // we will expand the `replace` template manually using the captured groups.
            
            // To do this simply, we parse the `replace` string for $N or ${N} and substitute them.
            let mut expanded = String::new();
            let mut chars = replace.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '$' {
                    let mut group_idx_str = String::new();
                    let mut is_braced = false;

                    if let Some(&'{') = chars.peek() {
                        is_braced = true;
                        chars.next(); // consume '{'
                    }

                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_ascii_digit() {
                            group_idx_str.push(next_c);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    if is_braced {
                        if let Some(&'}') = chars.peek() {
                            chars.next(); // consume '}'
                        }
                    }

                    if let Ok(group_idx) = group_idx_str.parse::<usize>() {
                        if let Some(matched_group) = caps.get(group_idx) {
                            let mut group_val = matched_group.as_str().to_string();
                            
                            // Apply modifier if configured
                            if let Some(modifier) = modifiers.get(&group_idx) {
                                match modifier.as_str() {
                                    "lowercase" => group_val = group_val.to_lowercase(),
                                    "uppercase" => group_val = group_val.to_uppercase(),
                                    "capitalize" => {
                                        let mut c = group_val.chars();
                                        group_val = match c.next() {
                                            None => String::new(),
                                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                        }
                                    }
                                    _ => {} // Unknown modifier, ignore
                                }
                            }
                            expanded.push_str(&group_val);
                        }
                    } else {
                        // Not a valid group ref, just output what we consumed
                        expanded.push('$');
                        if is_braced {
                            expanded.push('{');
                        }
                        expanded.push_str(&group_idx_str);
                    }
                } else {
                    expanded.push(c);
                }
            }
            expanded
        });

        ExtensionResult::Success(ExtensionOutput::Single(result.into_owned()))
    }
}

#[derive(Error, Debug)]
pub enum RegexTransformExtensionError {
    #[error("missing 'source' parameter")]
    MissingSourceParameter,

    #[error("missing 'find' parameter")]
    MissingFindParameter,

    #[error("missing 'replace' parameter")]
    MissingReplaceParameter,

    #[error("invalid regex parameter: {0}")]
    InvalidRegex(String),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn regex_transform_works_without_modifiers() {
        let extension = RegexTransformExtension::new();

        let param = vec![
            ("source".to_string(), Value::String("hello world".to_string())),
            ("find".to_string(), Value::String("(hello)".to_string())),
            ("replace".to_string(), Value::String("$1!".to_string())),
        ]
        .into_iter()
        .collect::<Params>();

        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("hello! world".to_string())
        );
    }
    
    #[test]
    fn regex_transform_with_modifiers() {
        let extension = RegexTransformExtension::new();

        let mut modifiers = HashMap::new();
        modifiers.insert("1".to_string(), Value::String("uppercase".to_string()));
        modifiers.insert("2".to_string(), Value::String("capitalize".to_string()));
        modifiers.insert("3".to_string(), Value::String("lowercase".to_string()));

        let param = vec![
            ("source".to_string(), Value::String("BAnk noteS".to_string())),
            ("find".to_string(), Value::String("^([A-Z])([A-Z])([a-z]+ note[A-Z])".to_string())),
            ("replace".to_string(), Value::String("${1}${2}${3}".to_string())),
            ("modifiers".to_string(), Value::Object(modifiers)),
        ]
        .into_iter()
        .collect::<Params>();

        // B -> B (uppercase)
        // A -> A (capitalize)
        // nk noteS -> nk notes (lowercase)
        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("BAnk notes".to_string())
        );
    }
    
    #[test]
    fn regex_transform_double_caps_autocorrect() {
        let extension = RegexTransformExtension::new();

        let mut modifiers = HashMap::new();
        modifiers.insert("2".to_string(), Value::String("lowercase".to_string()));

        let param = vec![
            ("source".to_string(), Value::String("BAnk".to_string())),
            ("find".to_string(), Value::String("^([A-Z])([A-Z])(.+)".to_string())),
            ("replace".to_string(), Value::String("$1$2$3".to_string())),
            ("modifiers".to_string(), Value::Object(modifiers)),
        ]
        .into_iter()
        .collect::<Params>();

        assert_eq!(
            extension
                .calculate(&crate::Context::default(), &HashMap::default(), &param)
                .into_success()
                .unwrap(),
            ExtensionOutput::Single("Bank".to_string())
        );
    }
}
