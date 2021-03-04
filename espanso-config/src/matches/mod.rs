use serde_yaml::Mapping;

use crate::counter::{next_id, StructId};

mod group;
mod store;

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
  cause: MatchCause,
  effect: MatchEffect,

  // Metadata
  label: Option<String>,

  // Internals
  _id: StructId,
}

impl Default for Match {
  fn default() -> Self {
    Self {
      cause: MatchCause::None,
      effect: MatchEffect::None,
      label: None,
      _id: next_id(),
    }
  }
}

// Causes

#[derive(Debug, Clone, PartialEq)]
pub enum MatchCause {
  None,
  Trigger(TriggerCause),
  // TODO: regex
  // TODO: shortcut
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerCause {
  pub triggers: Vec<String>,

  pub left_word: bool,
  pub right_word: bool,

  pub propagate_case: bool,
}

impl Default for TriggerCause {
  fn default() -> Self {
    Self {
      triggers: Vec::new(),
      left_word: false,
      right_word: false,
      propagate_case: false,
    }
  }
}

// Effects

#[derive(Debug, Clone, PartialEq)]
pub enum MatchEffect {
  None,
  Text(TextEffect),
  // TODO: image
  // TODO: rich text
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextEffect {
  pub replace: String,
  pub vars: Vec<Variable>,
}

impl Default for TextEffect {
  fn default() -> Self {
    Self {
      replace: String::new(),
      vars: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
  pub name: String,
  pub var_type: String,
  pub params: Mapping,

  // Internals
  _id: StructId,
}

impl Default for Variable {
  fn default() -> Self {
    Self {
      name: String::new(),
      var_type: String::new(),
      params: Mapping::new(),
      _id: next_id(),
    }
  }
}
