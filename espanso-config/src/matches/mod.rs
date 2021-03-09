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

use serde_yaml::Mapping;

use crate::counter::{next_id, StructId};

mod group;
pub mod store;

#[derive(Debug, Clone)]
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

impl PartialEq for Match {
  fn eq(&self, other: &Self) -> bool {
    self.cause == other.cause && self.effect == other.effect && self.label == other.label
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

#[derive(Debug, Clone)]
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

impl PartialEq for Variable {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.var_type == other.var_type && self.params == other.params
  }
}
