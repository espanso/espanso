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
use config::ConfigStore;
use matches::store::MatchStore;
use std::path::Path;
use thiserror::Error;

pub mod config;
mod counter;
pub mod error;
mod legacy;
pub mod matches;
mod util;

#[allow(clippy::type_complexity)]
pub fn load(
  base_path: &Path,
) -> Result<(
  Box<dyn ConfigStore>,
  Box<dyn MatchStore>,
  Vec<error::NonFatalErrorSet>,
)> {
  let config_dir = base_path.join("config");
  if !config_dir.exists() || !config_dir.is_dir() {
    return Err(ConfigError::MissingConfigDir().into());
  }

  let (config_store, non_fatal_config_errors) = config::load_store(&config_dir)?;
  let root_paths = config_store.get_all_match_paths();

  let (match_store, non_fatal_match_errors) =
    matches::store::load(&root_paths.into_iter().collect::<Vec<String>>());

  let mut non_fatal_errors = Vec::new();
  non_fatal_errors.extend(non_fatal_config_errors);
  non_fatal_errors.extend(non_fatal_match_errors);

  Ok((
    Box::new(config_store),
    Box::new(match_store),
    non_fatal_errors,
  ))
}

pub fn load_legacy(
  config_dir: &Path,
  package_dir: &Path,
) -> Result<(Box<dyn ConfigStore>, Box<dyn MatchStore>)> {
  legacy::load(config_dir, package_dir)
}

pub fn is_legacy_config(base_dir: &Path) -> bool {
  base_dir.join("user").is_dir() && base_dir.join("default.yml").is_file()
}

#[derive(Error, Debug)]
pub enum ConfigError {
  #[error("missing config directory")]
  MissingConfigDir(),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::tests::use_test_directory;
  use config::AppProperties;

  #[test]
  fn load_works_correctly() {
    use_test_directory(|base, match_dir, config_dir| {
      let base_file = match_dir.join("base.yml");
      std::fs::write(
        base_file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("another.yml");
      std::fs::write(
        another_file,
        r#"
      imports:
        - "_sub.yml"

      matches:
        - trigger: "hello2"
          replace: "world2"
      "#,
      )
      .unwrap();

      let under_file = match_dir.join("_sub.yml");
      std::fs::write(
        under_file,
        r#"
      matches:
        - trigger: "hello3"
          replace: "world3"
      "#,
      )
      .unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(config_file, "").unwrap();

      let custom_config_file = config_dir.join("custom.yml");
      std::fs::write(
        custom_config_file,
        r#"
      filter_title: "Chrome"

      use_standard_includes: false
      includes: ["../match/another.yml"]
      "#,
      )
      .unwrap();

      let (config_store, match_store, errors) = load(base).unwrap();

      assert_eq!(errors.len(), 0);
      assert_eq!(config_store.default().match_paths().len(), 2);
      assert_eq!(
        config_store
          .active(&AppProperties {
            title: Some("Google Chrome"),
            class: None,
            exec: None,
          })
          .match_paths()
          .len(),
        1
      );

      assert_eq!(
        match_store
          .query(config_store.default().match_paths())
          .matches
          .len(),
        3
      );
      assert_eq!(
        match_store
          .query(
            config_store
              .active(&AppProperties {
                title: Some("Chrome"),
                class: None,
                exec: None,
              })
              .match_paths()
          )
          .matches
          .len(),
        2
      );
    });
  }

  #[test]
  fn load_non_fatal_errors() {
    use_test_directory(|base, match_dir, config_dir| {
      let base_file = match_dir.join("base.yml");
      std::fs::write(
        base_file,
        r#"
      matches:
        - "invalid"invalid
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("another.yml");
      std::fs::write(
        another_file,
        r#"
      imports:
        - "_sub.yml"

      matches:
        - trigger: "hello2"
          replace: "world2"
      "#,
      )
      .unwrap();

      let under_file = match_dir.join("_sub.yml");
      std::fs::write(
        under_file,
        r#"
      matches:
        - trigger: "hello3"
          replace: "world3"invalid
      "#,
      )
      .unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(config_file, r"").unwrap();

      let custom_config_file = config_dir.join("custom.yml");
      std::fs::write(
        custom_config_file,
        r#"
      filter_title: "Chrome"
      "

      use_standard_includes: false
      includes: ["../match/another.yml"]
      "#,
      )
      .unwrap();

      let (config_store, match_store, errors) = load(base).unwrap();

      assert_eq!(errors.len(), 3);
      // It shouldn't have loaded the "config.yml" one because of the YAML error
      assert_eq!(config_store.configs().len(), 1);
      // It shouldn't load "base.yml" and "_sub.yml" due to YAML errors
      assert_eq!(match_store.loaded_paths().len(), 1);
    });
  }

  #[test]
  fn load_non_fatal_match_errors() {
    use_test_directory(|base, match_dir, config_dir| {
      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world"
        - trigger: "invalid because there is no action field"
      "#,
      )
      .unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(config_file, r"").unwrap();

      let (config_store, match_store, errors) = load(base).unwrap();

      assert_eq!(errors.len(), 1);
      assert_eq!(errors[0].file, base_file);
      assert_eq!(errors[0].errors.len(), 1);

      assert_eq!(
        match_store
          .query(config_store.default().match_paths())
          .matches
          .len(),
        1
      );
    });
  }

  #[test]
  fn load_fatal_errors() {
    use_test_directory(|base, match_dir, config_dir| {
      let base_file = match_dir.join("base.yml");
      std::fs::write(
        base_file,
        r"
      matches:
        - trigger: hello
          replace: world
      ",
      )
      .unwrap();

      let config_file = config_dir.join("default.yml");
      std::fs::write(
        config_file,
        r#"
      invalid

      "
      "#,
      )
      .unwrap();

      // A syntax error in the default.yml file cannot be handled gracefully
      assert!(load(base).is_err());
    });
  }

  #[test]
  fn load_without_valid_config_dir() {
    use_test_directory(|_, match_dir, _| {
      // To correcly load the configs, the "load" method looks for the "config" directory
      assert!(load(match_dir).is_err());
    });
  }
}
