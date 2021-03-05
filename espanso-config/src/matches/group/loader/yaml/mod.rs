use crate::matches::{Match, Variable, group::{MatchGroup, path::resolve_imports}};
use log::warn;
use parse::YAMLMatchGroup;
use anyhow::Result;
use std::convert::{TryFrom, TryInto};

use self::parse::{YAMLMatch, YAMLVariable};
use crate::matches::{MatchCause, MatchEffect, TextEffect, TriggerCause};

use super::Importer;

mod parse;

pub(crate) struct YAMLImporter {}

impl YAMLImporter {
  pub fn new() -> Self {
    Self {}
  }
}

impl Importer for YAMLImporter {
  fn is_supported(&self, extension: &str) -> bool {
    extension == "yaml" || extension == "yml"
  }

  // TODO: test
  // TODO: test resolve imports
  // TODO: test cyclical dependency
  fn load_group(
    &self,
    path: &std::path::Path,
  ) -> anyhow::Result<crate::matches::group::MatchGroup> {
    let yaml_group = YAMLMatchGroup::parse_from_file(path)?;

    let global_vars: Result<Vec<Variable>> = yaml_group
      .global_vars
      .as_ref()
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|var| var.clone().try_into())
      .collect();

    let matches: Result<Vec<Match>> = yaml_group
      .matches
      .as_ref()
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|m| m.clone().try_into())
      .collect();

    // Resolve imports
    let resolved_imports = resolve_imports(path, &yaml_group.imports.unwrap_or_default())?;

    Ok(MatchGroup {
      imports: resolved_imports,
      global_vars: global_vars?,
      matches: matches?,
    })
  }
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

    if let MatchEffect::None = effect {
      warn!("match caused by {:?} does not produce any effect. Did you forget the 'replace' field?", cause);
    }

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

#[cfg(test)]
mod tests {
  use serde_yaml::{Mapping, Value};

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
