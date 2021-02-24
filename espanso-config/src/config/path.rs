use std::collections::HashSet;

use glob::glob;
use log::error;

// TODO: test
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
            Err(err) => {
              error!("glob error when processing pattern: {}, with error: {}", glob_pattern, err)
            }
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
