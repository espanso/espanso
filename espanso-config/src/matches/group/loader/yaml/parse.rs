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

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use crate::util::is_yaml_empty;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatchGroup {
  #[serde(default)]
  pub imports: Option<Vec<String>>,

  #[serde(default)]
  pub global_vars: Option<Vec<YAMLVariable>>,

  #[serde(default)]
  pub matches: Option<Vec<YAMLMatch>>,
}

impl YAMLMatchGroup {
  pub fn parse_from_str(yaml: &str) -> Result<Self> {
    // Because an empty string is not valid YAML but we want to support it anyway
    if is_yaml_empty(yaml) {
      return Ok(serde_yaml::from_str(
        "arbitrary_field_that_will_not_block_the_parser: true",
      )?);
    }

    Ok(serde_yaml::from_str(yaml)?)
  }

  // TODO: test
  pub fn parse_from_file(path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(path)?;
    Self::parse_from_str(&content)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatch {
  #[serde(default)]
  pub trigger: Option<String>,

  #[serde(default)]
  pub triggers: Option<Vec<String>>,

  #[serde(default)]
  pub replace: Option<String>,

  #[serde(default)]
  pub image_path: Option<String>, // TODO: map

  #[serde(default)]
  pub form: Option<String>,

  #[serde(default)]
  pub form_fields: Option<Mapping>,

  #[serde(default)]
  pub vars: Option<Vec<YAMLVariable>>,

  #[serde(default)]
  pub word: Option<bool>,

  #[serde(default)]
  pub left_word: Option<bool>,

  #[serde(default)]
  pub right_word: Option<bool>,

  #[serde(default)]
  pub propagate_case: Option<bool>,

  #[serde(default)]
  pub uppercase_style: Option<String>,

  #[serde(default)]
  pub force_clipboard: Option<bool>,

  #[serde(default)]
  pub markdown: Option<String>,

  #[serde(default)]
  pub paragraph: Option<bool>,

  #[serde(default)]
  pub html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct YAMLVariable {
  pub name: String,

  #[serde(rename = "type")]
  pub var_type: String,

  #[serde(default = "default_params")]
  pub params: Mapping,
}

fn default_params() -> Mapping {
  Mapping::new()
}
