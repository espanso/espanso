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

use std::collections::HashMap;

use event::Event;

pub mod event;
pub mod regex;
pub mod rolling;
mod util;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult<Id> {
  pub id: Id,
  pub trigger: String,
  pub left_separator: Option<String>,
  pub right_separator: Option<String>,
  pub vars: HashMap<String, String>,
}

impl<Id: Default> Default for MatchResult<Id> {
  fn default() -> Self {
    Self {
      id: Id::default(),
      trigger: "".to_string(),
      left_separator: None,
      right_separator: None,
      vars: HashMap::new(),
    }
  }
}

pub trait Matcher<'a, State, Id>
where
  Id: Clone,
{
  fn process(&'a self, prev_state: Option<&State>, event: Event) -> (State, Vec<MatchResult<Id>>);
}
