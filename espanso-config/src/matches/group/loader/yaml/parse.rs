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
  pub form: Option<String>, // TODO: map

  #[serde(default)]
  pub form_fields: Option<HashMap<String, Value>>, // TODO: map

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
  pub force_clipboard: Option<bool>,

  #[serde(default)]
  pub markdown: Option<String>, // TODO: map

  #[serde(default)]
  pub paragraph: Option<bool>, // TODO: map

  #[serde(default)]
  pub html: Option<String>, // TODO: map
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