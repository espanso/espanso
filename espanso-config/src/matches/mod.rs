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

use std::collections::{BTreeMap};
use enum_as_inner::EnumAsInner;
use ordered_float::OrderedFloat;

use crate::counter::{StructId};

pub(crate) mod group;
pub mod store;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Match {
  pub id: StructId,

  pub cause: MatchCause,
  pub effect: MatchEffect,

  // Metadata
  pub label: Option<String>,
}

impl Default for Match {
  fn default() -> Self {
    Self {
      cause: MatchCause::None,
      effect: MatchEffect::None,
      label: None,
      id: 0,
    }
  }
}

// Causes

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum MatchCause {
  None,
  Trigger(TriggerCause),
  // TODO: regex
  // TODO: shortcut
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MatchEffect {
  None,
  Text(TextEffect),
  // TODO: image
  // TODO: rich text
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
  pub id: StructId,
  pub name: String,
  pub var_type: String,
  pub params: Params,
}

impl Default for Variable {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::new(),
      var_type: String::new(),
      params: Params::new(),
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