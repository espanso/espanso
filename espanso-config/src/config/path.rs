use std::collections::HashSet;

use glob::glob;
use log::error;

pub fn calculate_paths<'a>(glob_patterns: impl Iterator<Item = &'a String>) -> HashSet<String> {
  let mut path_set = HashSet::new();
  for glob_pattern in glob_patterns {
    let entries = glob(glob_pattern);
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
mod tests {
  use std::fs::create_dir_all;

  use super::*;
  use tempdir::TempDir;

  #[test]
  fn calculate_paths_works_correctly() {
    let dir = TempDir::new("tempconfig").unwrap();
    let match_dir = dir.path().join("match");
    create_dir_all(&match_dir).unwrap();

    let sub_dir = match_dir.join("sub");
    create_dir_all(&sub_dir).unwrap();

    std::fs::write(match_dir.join("base.yml"), "test").unwrap();
    std::fs::write(match_dir.join("another.yml"), "test").unwrap();
    std::fs::write(match_dir.join("_sub.yml"), "test").unwrap();
    std::fs::write(sub_dir.join("sub.yml"), "test").unwrap();

    let result = calculate_paths(vec![
      format!("{}/**/*.yml", dir.path().to_string_lossy()),
      format!("{}/match/sub/*.yml", dir.path().to_string_lossy()),
      // Invalid path
      "invalid".to_string(),
    ].iter());

    let mut expected = HashSet::new();
    expected.insert(format!("{}/match/base.yml", dir.path().to_string_lossy()));
    expected.insert(format!("{}/match/another.yml", dir.path().to_string_lossy()));
    expected.insert(format!("{}/match/_sub.yml", dir.path().to_string_lossy()));
    expected.insert(format!("{}/match/sub/sub.yml", dir.path().to_string_lossy()));

    assert_eq!(result, expected);
  }
}
