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

use std::collections::HashSet;
use super::VAR_REGEX;

pub(crate) fn get_body_variable_names(body: &str) -> HashSet<&str> {
  let mut variables = HashSet::new();
  for caps in VAR_REGEX.captures_iter(&body) {
    let var_name = caps.name("name").unwrap().as_str();
    variables.insert(var_name);
  }
  variables
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::iter::FromIterator;

  #[test]
  fn get_body_variable_names_no_vars() {
    assert_eq!(
      get_body_variable_names("no variables"),
      HashSet::from_iter(vec![]),
    );
  }

  #[test]
  fn get_body_variable_names_multiple_vars() {
    assert_eq!(
      get_body_variable_names("hello {{world}} name {{greet}}"),
      HashSet::from_iter(vec!["world", "greet"]),
    );
  }
}