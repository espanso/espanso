/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn default_title() -> String {
    "espanso".to_owned()
}

fn default_icon() -> Option<String> {
    None
}

fn default_fields() -> HashMap<String, FieldConfig> {
    HashMap::new()
}

fn default_max_form_width() -> i32 {
    700
}

fn default_max_form_height() -> i32 {
    500
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormConfig {
    #[serde(default = "default_title")]
    pub title: String,

    #[serde(default = "default_icon")]
    pub icon: Option<String>,

    pub layout: String,

    #[serde(default = "default_fields")]
    pub fields: HashMap<String, FieldConfig>,

    #[serde(default = "default_max_form_width")]
    pub max_form_width: i32,

    #[serde(default = "default_max_form_height")]
    pub max_form_height: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldConfig {
    pub field_type: FieldTypeConfig,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            field_type: FieldTypeConfig::Text(TextFieldConfig::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FieldTypeConfig {
    Text(TextFieldConfig),
    Choice(ChoiceFieldConfig),
    List(ListFieldConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextFieldConfig {
    pub default: String,
    pub multiline: bool,
}

/// A choice/list value. `Plain` means label and id are the same string;
/// `LabelId` separates the display text (`label`) from the output value (`id`).
#[derive(Debug, Serialize, Clone)]
pub enum ChoiceValue {
    Plain(String),
    LabelId { label: String, id: String },
}

impl ChoiceValue {
    pub fn label(&self) -> &str {
        match self {
            ChoiceValue::Plain(s) => s,
            ChoiceValue::LabelId { label, .. } => label,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            ChoiceValue::Plain(s) => s,
            ChoiceValue::LabelId { id, .. } => id,
        }
    }
}

impl<'de> Deserialize<'de> for ChoiceValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        struct ChoiceValueVisitor;

        impl<'de> de::Visitor<'de> for ChoiceValueVisitor {
            type Value = ChoiceValue;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or an object with 'id' and 'label'")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<ChoiceValue, E> {
                Ok(ChoiceValue::Plain(v.to_owned()))
            }

            fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<ChoiceValue, M::Error> {
                let mut id = None;
                let mut label = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "label" => label = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                let id: String = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let label: String = label.ok_or_else(|| de::Error::missing_field("label"))?;
                Ok(ChoiceValue::LabelId { label, id })
            }
        }

        deserializer.deserialize_any(ChoiceValueVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChoiceFieldConfig {
    pub values: Vec<ChoiceValue>,
    pub default: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListFieldConfig {
    pub values: Vec<ChoiceValue>,
    pub default: String,
    pub separator: String,
}

impl<'de> serde::Deserialize<'de> for FieldConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let auto_field = AutoFieldConfig::deserialize(deserializer)?;
        Ok(FieldConfig::from(&auto_field))
    }
}

impl<'a> From<&'a AutoFieldConfig> for FieldConfig {
    fn from(other: &'a AutoFieldConfig) -> Self {
        let field_type = match other.field_type.as_ref() {
            "text" => {
                let mut config = TextFieldConfig::default();

                if let Some(default) = &other.default {
                    config.default.clone_from(default);
                }

                config.multiline = other.multiline;

                FieldTypeConfig::Text(config)
            }
            "choice" => {
                let mut config = ChoiceFieldConfig {
                    values: other.values.clone(),
                    ..Default::default()
                };

                if let Some(default) = &other.default {
                    config.default.clone_from(default);
                }

                FieldTypeConfig::Choice(config)
            }
            "list" => {
                let mut config = ListFieldConfig {
                    values: other.values.clone(),
                    ..Default::default()
                };

                if let Some(default) = &other.default {
                    config.default.clone_from(default);
                }

                config.separator.clone_from(&other.separator);

                FieldTypeConfig::List(config)
            }
            _ => {
                panic!("invalid field type: {}", other.field_type);
            }
        };

        Self { field_type }
    }
}

fn default_type() -> String {
    "text".to_owned()
}

fn default_default() -> Option<String> {
    None
}

fn default_multiline() -> bool {
    false
}

fn default_values() -> Vec<ChoiceValue> {
    Vec::new()
}

fn default_separator() -> String {
    ", ".to_owned()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoFieldConfig {
    #[serde(rename = "type", default = "default_type")]
    pub field_type: String,

    #[serde(default = "default_default")]
    pub default: Option<String>,

    #[serde(default = "default_multiline")]
    pub multiline: bool,

    #[serde(default = "default_values")]
    pub values: Vec<ChoiceValue>,

    #[serde(default = "default_separator")]
    pub separator: String,
}
