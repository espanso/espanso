use std::{collections::HashSet, path::{Path, PathBuf}};

use glob::glob;
use log::error;
use regex::Regex;

lazy_static! {
  static ref ABSOLUTE_PATH: Regex = Regex::new(r"(?m)^([a-zA-Z]:/|/).*$").unwrap();
}

pub fn calculate_paths<'a>(base_dir: &Path, glob_patterns: impl Iterator<Item = &'a String>) -> HashSet<String> {
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
              path_set.insert(path.to_string_lossy().to_string());
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
  use std::{fs::create_dir_all};

  #[test]
  fn calculate_paths_relative_paths() {
    use_test_directory(|base, match_dir, config_dir| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      std::fs::write(match_dir.join("base.yml"), "test").unwrap();
      std::fs::write(match_dir.join("another.yml"), "test").unwrap();
      std::fs::write(match_dir.join("_sub.yml"), "test").unwrap();
      std::fs::write(sub_dir.join("sub.yml"), "test").unwrap();

      let result = calculate_paths(base, vec![
        "**/*.yml".to_string(),
        "match/sub/*.yml".to_string(),
        // Invalid path
        "invalid".to_string(),
      ].iter());

      let mut expected = HashSet::new();
      expected.insert(format!("{}/match/base.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/another.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/_sub.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/sub/sub.yml", base.to_string_lossy()));

      assert_eq!(result, expected);
    });
  }

  #[test]
  fn calculate_paths_absolute_paths() {
    use_test_directory(|base, match_dir, config_dir| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      std::fs::write(match_dir.join("base.yml"), "test").unwrap();
      std::fs::write(match_dir.join("another.yml"), "test").unwrap();
      std::fs::write(match_dir.join("_sub.yml"), "test").unwrap();
      std::fs::write(sub_dir.join("sub.yml"), "test").unwrap();

      let result = calculate_paths(base, vec![
        format!("{}/**/*.yml", base.to_string_lossy()),
        format!("{}/match/sub/*.yml", base.to_string_lossy()),
        // Invalid path
        "invalid".to_string(),
      ].iter());

      let mut expected = HashSet::new();
      expected.insert(format!("{}/match/base.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/another.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/_sub.yml", base.to_string_lossy()));
      expected.insert(format!("{}/match/sub/sub.yml", base.to_string_lossy()));

      assert_eq!(result, expected);
    });
  }
}
