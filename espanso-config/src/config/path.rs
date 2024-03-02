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

use std::{collections::HashSet, path::Path};

use glob::glob;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;

lazy_static! {
  static ref ABSOLUTE_PATH: Regex = Regex::new(r"(?m)^([a-zA-Z]:\\|/).*$").unwrap();
}

pub fn calculate_paths<'a>(
  base_dir: &Path,
  glob_patterns: impl Iterator<Item = &'a String>,
) -> HashSet<String> {
  let mut path_set = HashSet::new();
  for glob_pattern in glob_patterns {
    // Handle relative and absolute paths appropriately
    let pattern = if ABSOLUTE_PATH.is_match(glob_pattern) {
      glob_pattern.clone()
    } else {
      format!("{}/{}", base_dir.to_string_lossy(), glob_pattern)
    };

    let entries = glob(&pattern);
    match entries {
      Ok(paths) => {
        for path in paths {
          match path {
            Ok(path) => {
              // Canonicalize the path
              match dunce::canonicalize(&path) {
                Ok(canonical_path) => {
                  path_set.insert(canonical_path.to_string_lossy().to_string());
                }
                Err(err) => {
                  error!(
                    "unable to canonicalize path from glob: {:?}, with error: {}",
                    path, err
                  );
                }
              }
            }
            Err(err) => error!(
              "glob error when processing pattern: {}, with error: {}",
              glob_pattern, err
            ),
          }
        }
      }
      Err(err) => {
        error!(
          "unable to calculate glob from pattern: {}, with error: {}",
          glob_pattern, err
        );
      }
    }
  }

  path_set
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::util::tests::use_test_directory;
  use std::fs::create_dir_all;

  #[test]
  fn calculate_paths_relative_paths() {
    use_test_directory(|base, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();
      let another_file = match_dir.join("another.yml");
      std::fs::write(&another_file, "test").unwrap();
      let under_file = match_dir.join("_sub.yml");
      std::fs::write(&under_file, "test").unwrap();
      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let result = calculate_paths(
        base,
        [
          "**/*.yml".to_string(),
          "match/sub/*.yml".to_string(),
          // Invalid path
          "invalid".to_string(),
        ]
        .iter(),
      );

      let mut expected = HashSet::new();
      expected.insert(base_file.to_string_lossy().to_string());
      expected.insert(another_file.to_string_lossy().to_string());
      expected.insert(under_file.to_string_lossy().to_string());
      expected.insert(sub_file.to_string_lossy().to_string());

      assert_eq!(result, expected);
    });
  }

  #[test]
  fn calculate_paths_relative_with_parent_modifier() {
    use_test_directory(|base, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(base_file, "test").unwrap();
      let another_file = match_dir.join("another.yml");
      std::fs::write(another_file, "test").unwrap();
      let under_file = match_dir.join("_sub.yml");
      std::fs::write(under_file, "test").unwrap();
      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let result = calculate_paths(base, ["match/sub/../sub/*.yml".to_string()].iter());

      let mut expected = HashSet::new();
      expected.insert(sub_file.to_string_lossy().to_string());

      assert_eq!(result, expected);
    });
  }

  #[test]
  fn calculate_paths_absolute_paths() {
    use_test_directory(|base, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(&base_file, "test").unwrap();
      let another_file = match_dir.join("another.yml");
      std::fs::write(&another_file, "test").unwrap();
      let under_file = match_dir.join("_sub.yml");
      std::fs::write(&under_file, "test").unwrap();
      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(&sub_file, "test").unwrap();

      let result = calculate_paths(
        base,
        [
          format!("{}/**/*.yml", base.to_string_lossy()),
          format!("{}/match/sub/*.yml", base.to_string_lossy()),
          // Invalid path
          "invalid".to_string(),
        ]
        .iter(),
      );

      let mut expected = HashSet::new();
      expected.insert(base_file.to_string_lossy().to_string());
      expected.insert(another_file.to_string_lossy().to_string());
      expected.insert(under_file.to_string_lossy().to_string());
      expected.insert(sub_file.to_string_lossy().to_string());

      assert_eq!(result, expected);
    });
  }
}
