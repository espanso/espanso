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

#[macro_use]
extern crate lazy_static;

#[macro_use]
#[cfg(test)]
extern crate include_dir;

#[macro_use]
#[cfg(test)]
extern crate test_case;

use anyhow::Result;
use thiserror::Error;

// TODO: implement
// Use yaml-rust with "preserve-order" = true
// Strategy:
// 1. Backup the current config directory in a zip archive (also with the packages)
// 2. Create a temporary directory alonside the legacy one called "espanso-new"
// 3. Convert all the files and write the output into "espanso-new"
// 4. Rename the legacy dir to "espanso-old"
// 5. Rename new dir to "espanso"
// 6. If the legacy directory was a symlink, try to recreate it (ask the user first)

// TODO: before attempting the migration strategy, check if the current
// espanso config directory is a symlink and, if so, attempt to remap
// the symlink with the new dir (after asking the user)
// This is necessary because in order to be safe, the migration strategy
// creates the new config on a new temporary directory and then "swaps"
// the old with the new one

// TODO: test case with packages

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;
  use include_dir::{include_dir, Dir};

  static BASE_CASE: Dir = include_dir!("test/base");

  #[test_case(&BASE_CASE; "base case")]
  fn test_migration(test_data: &Dir) {
    // TODO
    assert!(false);
  }
}
