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

use enum_as_inner::EnumAsInner;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

use crate::counter::StructId;

pub(crate) mod group;
pub mod store;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Match {
  pub id: StructId,

  pub cause: MatchCause,
  pub effect: MatchEffect,

  // Metadata
  pub label: Option<String>,
  pub search_terms: Vec<String>,
}

impl Default for Match {
  fn default() -> Self {
    Self {
      cause: MatchCause::None,
      effect: MatchEffect::None,
      label: None,
      id: 0,
      search_terms: vec![],
    }
  }
}

impl Match {
  // TODO: test
  pub fn description(&self) -> &str {
    if let Some(label) = &self.label {
      label
    } else if let MatchEffect::Text(text_effect) = &self.effect {
      &text_effect.replace
    } else if let MatchEffect::Image(_) = &self.effect {
      "Image content"
    } else {
      "No description available for this match"
    }
  }

  // TODO: test
  pub fn cause_description(&self) -> Option<&str> {
    self.cause.description()
  }

  pub fn search_terms(&self) -> Vec<&str> {
    self
      .search_terms
      .iter()
      .map(String::as_str)
      .chain(self.cause.search_terms())
      .collect()
  }
}

// Causes

#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumAsInner)]
pub enum MatchCause {
  None,
  Trigger(TriggerCause),
  Regex(RegexCause),
  // TODO: shortcut
}

impl MatchCause {
  pub fn description(&self) -> Option<&str> {
    match &self {
      MatchCause::Trigger(trigger_cause) => trigger_cause.triggers.first().map(String::as_str),
      MatchCause::Regex(trigger_cause) => Some(trigger_cause.regex.as_str()),
      MatchCause::None => None,
    }
    // TODO: insert rendering for hotkey/shortcut
  }

  pub fn long_description(&self) -> String {
    match &self {
      MatchCause::Trigger(trigger_cause) => format!("triggers: {:?}", trigger_cause.triggers),
      MatchCause::Regex(trigger_cause) => format!("regex: {:?}", trigger_cause.regex),
      MatchCause::None => "No description available".to_owned(),
    }
    // TODO: insert rendering for hotkey/shortcut
  }

  pub fn search_terms(&self) -> Vec<&str> {
    if let MatchCause::Trigger(trigger_cause) = &self {
      trigger_cause.triggers.iter().map(String::as_str).collect()
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TriggerCause {
  pub triggers: Vec<String>,

  pub left_word: bool,
  pub right_word: bool,

  pub propagate_case: bool,
  pub uppercase_style: UpperCasingStyle,
}

impl Default for TriggerCause {
  fn default() -> Self {
    Self {
      triggers: Vec::new(),
      left_word: false,
      right_word: false,
      propagate_case: false,
      uppercase_style: UpperCasingStyle::Uppercase,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpperCasingStyle {
  Uppercase,
  Capitalize,
  CapitalizeWords,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RegexCause {
  pub regex: String,
}

// Effects

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumAsInner)]
pub enum MatchEffect {
  None,
  Text(TextEffect),
  Image(ImageEffect),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextEffect {
  pub replace: String,
  pub vars: Vec<Variable>,
  pub format: TextFormat,
  pub force_mode: Option<TextInjectMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextFormat {
  Plain,
  Markdown,
  Html,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextInjectMode {
  Keys,
  Clipboard,
}

impl Default for TextEffect {
  fn default() -> Self {
    Self {
      replace: String::new(),
      vars: Vec::new(),
      format: TextFormat::Plain,
      force_mode: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ImageEffect {
  pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
  pub id: StructId,
  pub name: String,
  pub var_type: String,
  pub params: Params,
  pub inject_vars: bool,
  pub depends_on: Vec<String>,
}

impl Default for Variable {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::new(),
      var_type: String::new(),
      params: Params::new(),
      inject_vars: true,
      depends_on: Vec::new(),
    }
  }
}

pub type Params = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumAsInner)]
pub enum Value {
  Null,
  Bool(bool),
  Number(Number),
  String(String),
  Array(Vec<Value>),
  Object(Params),
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Number {
  Integer(i64),
  Float(OrderedFloat<f64>),
}

#[cfg(test)]
mod tests {
  use super::*;

  fn trigger_cause() -> TriggerCause {
    TriggerCause {
      triggers: vec![":greet".to_string()],
      ..TriggerCause::default()
    }
  }

  fn regex_cause() -> RegexCause {
    RegexCause {
      regex: ":greet\\d".to_string(),
    }
  }

  #[test]
  fn match_cause_trigger_description() {
    let trigger = trigger_cause();

    assert_eq!(MatchCause::Trigger(trigger).description(), Some(":greet"));
  }

  #[test]
  fn match_cause_regex_description() {
    let regex = regex_cause();
    assert_eq!(MatchCause::Regex(regex).description(), Some(":greet\\d"));
  }

  #[test]
  fn match_cause_trigger_long_description() {
    let trigger = trigger_cause();

    assert_eq!(
      MatchCause::Trigger(trigger).long_description(),
      r#"triggers: [":greet"]"#
    );
  }

  #[test]
  fn match_cause_regex_long_description() {
    let regex = regex_cause();

    assert_eq!(
      MatchCause::Regex(regex).long_description(),
      r#"regex: ":greet\\d""#
    );
  }
}
