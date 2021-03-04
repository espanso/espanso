use std::{collections::HashMap, convert::{TryFrom, TryInto}, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use thiserror::Error;

use crate::util::is_yaml_empty;

use crate::matches::{Match, MatchCause, MatchEffect, TextEffect, TriggerCause, Variable};
use super::{MatchGroup};

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

impl TryFrom<YAMLMatchGroup> for MatchGroup {
  type Error = anyhow::Error;

  // TODO: test
  fn try_from(yaml_match_group: YAMLMatchGroup) -> Result<Self, Self::Error> {
    let global_vars: Result<Vec<Variable>> = yaml_match_group
      .global_vars
      .as_ref()
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|var| var.clone().try_into())
      .collect();

    let matches: Result<Vec<Match>> = yaml_match_group
      .matches
      .as_ref()
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|m| m.clone().try_into())
      .collect();

    Ok(MatchGroup {
      imports: yaml_match_group.imports.unwrap_or_default(),
      global_vars: global_vars?,
      matches: matches?,
      ..Default::default()
    })
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

impl TryFrom<YAMLMatch> for Match {
  type Error = anyhow::Error;

  // TODO: test
  fn try_from(yaml_match: YAMLMatch) -> Result<Self, Self::Error> {
    let triggers = if let Some(trigger) = yaml_match.trigger {
      Some(vec![trigger])
    } else if let Some(triggers) = yaml_match.triggers {
      Some(triggers)
    } else {
      None
    };

    let cause = if let Some(triggers) = triggers {
      MatchCause::Trigger(TriggerCause {
        triggers,
        left_word: yaml_match
          .left_word
          .or(yaml_match.word)
          .unwrap_or(TriggerCause::default().left_word),
        right_word: yaml_match
          .right_word
          .or(yaml_match.word)
          .unwrap_or(TriggerCause::default().right_word),
        propagate_case: yaml_match
          .propagate_case
          .unwrap_or(TriggerCause::default().propagate_case),
      })
    } else {
      MatchCause::None
    };

    let effect = if let Some(replace) = yaml_match.replace {
      let vars: Result<Vec<Variable>> = yaml_match
        .vars
        .unwrap_or_default()
        .into_iter()
        .map(|var| var.try_into())
        .collect();
      MatchEffect::Text(TextEffect {
        replace,
        vars: vars?,
      })
    } else {
      MatchEffect::None
    };

    // TODO: log none match effect

    Ok(Self {
      cause,
      effect,
      label: None,
      ..Default::default()
    })
  }
}

impl TryFrom<YAMLVariable> for Variable {
  type Error = anyhow::Error;

  // TODO: test
  fn try_from(yaml_var: YAMLVariable) -> Result<Self, Self::Error> {
    Ok(Self {
      name: yaml_var.name,
      var_type: yaml_var.var_type,
      params: yaml_var.params,
      ..Default::default()
    })
  }
}

#[derive(Error, Debug)]
pub enum YAMLConversionError {
  // TODO
//#[error("unknown data store error")]
//Unknown,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::matches::Match;

  fn create_match(yaml: &str) -> Result<Match> {
    let yaml_match: YAMLMatch = serde_yaml::from_str(yaml)?;
    let m: Match = yaml_match.try_into()?;
    Ok(m)
  }

  #[test]
  fn basic_match_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn multiple_triggers_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        triggers: ["Hello", "john"]
        replace: "world"
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string(), "john".to_string()],
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn word_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        word: true
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          left_word: true,
          right_word: true,
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn left_word_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        left_word: true
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          left_word: true,
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn right_word_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        right_word: true
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          right_word: true,
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn propagate_case_maps_correctly() {
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        propagate_case: true
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          propagate_case: true,
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          ..Default::default()
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn vars_maps_correctly() {
    let mut params = Mapping::new();
    params.insert(Value::String("param1".to_string()), Value::Bool(true));
    let vars = vec![Variable {
      name: "var1".to_string(),
      var_type: "test".to_string(),
      params,
      ..Default::default()
    }];
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        vars:
          - name: var1
            type: test
            params:
              param1: true
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          vars,
        }),
        ..Default::default()
      }
    )
  }

  #[test]
  fn vars_no_params_maps_correctly() {
    let vars = vec![Variable {
      name: "var1".to_string(),
      var_type: "test".to_string(),
      params: Mapping::new(),
      ..Default::default()
    }];
    assert_eq!(
      create_match(
        r#"
        trigger: "Hello"
        replace: "world"
        vars:
          - name: var1
            type: test
        "#
      )
      .unwrap(),
      Match {
        cause: MatchCause::Trigger(TriggerCause {
          triggers: vec!["Hello".to_string()],
          ..Default::default()
        }),
        effect: MatchEffect::Text(TextEffect {
          replace: "world".to_string(),
          vars,
        }),
        ..Default::default()
      }
    )
  }
}
