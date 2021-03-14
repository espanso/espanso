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

/// Check if the given string represents an empty YAML.
/// In other words, it checks if the document is only composed
/// of spaces and/or comments
pub fn is_yaml_empty(yaml: &str) -> bool {
  for line in yaml.lines() {
    let trimmed_line = line.trim();
    if !trimmed_line.starts_with("#") && !trimmed_line.is_empty() {
      return false;
    }
  }

  true
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use std::{fs::create_dir_all, path::Path};
  use tempdir::TempDir;

  pub fn use_test_directory(callback: impl FnOnce(&Path, &Path, &Path)) {
    let dir = TempDir::new("tempconfig").unwrap();
    let match_dir = dir.path().join("match");
    create_dir_all(&match_dir).unwrap();

    let config_dir = dir.path().join("config");
    create_dir_all(&config_dir).unwrap();

    callback(
      &dunce::canonicalize(&dir.path()).unwrap(),
      &dunce::canonicalize(match_dir).unwrap(),
      &dunce::canonicalize(config_dir).unwrap(),
    );
  }

  #[test]
  fn is_yaml_empty_document_empty() {
    assert_eq!(is_yaml_empty(""), true);
  }

  #[test]
  fn is_yaml_empty_document_with_comments() {
    assert_eq!(is_yaml_empty("\n#comment \n \n"), true);
  }

  #[test]
  fn is_yaml_empty_document_with_comments_and_content() {
    assert_eq!(is_yaml_empty("\n#comment \n field: true\n"), false);
  }

  #[test]
  fn is_yaml_empty_document_with_content() {
    assert_eq!(is_yaml_empty("\nfield: true\n"), false);
  }
}
