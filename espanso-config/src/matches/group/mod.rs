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

use anyhow::Result;
use std::path::Path;

use crate::error::NonFatalErrorSet;

use super::{Match, Variable};

pub(crate) mod loader;
mod path;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatchGroup {
  pub imports: Vec<String>,
  pub global_vars: Vec<Variable>,
  pub matches: Vec<Match>,
}

impl Default for MatchGroup {
  fn default() -> Self {
    Self {
      imports: Vec::new(),
      global_vars: Vec::new(),
      matches: Vec::new(),
    }
  }
}

impl MatchGroup {
  // TODO: test
  pub fn load(group_path: &Path) -> Result<(Self, Option<NonFatalErrorSet>)> {
    loader::load_match_group(group_path)
  }
}
