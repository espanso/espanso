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

use super::{Match, Variable};

mod default;

pub trait MatchStore {
  fn load(&mut self, paths: &[String]);
  fn query(&self, paths: &[String]) -> MatchSet;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchSet<'a> {
  pub matches: Vec<&'a Match>,
  pub global_vars: Vec<&'a Variable>,
}

pub fn new() -> impl MatchStore {
  // TODO: here we can replace the DefaultMatchStore with a caching wrapper
  // that returns the same response for the given "paths" query
  default::DefaultMatchStore::new()
}
