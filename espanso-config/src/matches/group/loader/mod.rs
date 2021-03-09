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
use thiserror::Error;

use self::yaml::YAMLImporter;

use super::MatchGroup;

mod yaml;

trait Importer {
  fn is_supported(&self, extension: &str) -> bool;
  fn load_group(&self, path: &Path) -> Result<MatchGroup>;
}

lazy_static! {
  static ref IMPORTERS: Vec<Box<dyn Importer + Sync + Send>> = vec![Box::new(YAMLImporter::new()),];
}

pub(crate) fn load_match_group(path: &Path) -> Result<MatchGroup> {
  if let Some(extension) = path.extension() {
    let extension = extension.to_string_lossy().to_lowercase();

    let importer = IMPORTERS
      .iter()
      .find(|importer| importer.is_supported(&extension));

    match importer {
      Some(importer) => match importer.load_group(path) {
        Ok(group) => Ok(group),
        Err(err) => Err(LoadError::ParsingError(err).into()),
      },
      None => Err(LoadError::InvalidFormat().into()),
    }
  } else {
    Err(LoadError::MissingExtension().into())
  }
}

#[derive(Error, Debug)]
pub enum LoadError {
  #[error("missing extension in match group file")]
  MissingExtension(),

  #[error("invalid match group format")]
  InvalidFormat(),

  #[error("parser reported an error: `{0}`")]
  ParsingError(anyhow::Error),
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::tests::use_test_directory;

  #[test]
  fn load_group_invalid_format() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base.invalid");
      std::fs::write(&file, "test").unwrap();

      assert!(matches!(
        load_match_group(&file)
          .unwrap_err()
          .downcast::<LoadError>()
          .unwrap(),
        LoadError::InvalidFormat()
      ));
    });
  }

  #[test]
  fn load_group_missing_extension() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base");
      std::fs::write(&file, "test").unwrap();

      assert!(matches!(
        load_match_group(&file)
          .unwrap_err()
          .downcast::<LoadError>()
          .unwrap(),
        LoadError::MissingExtension()
      ));
    });
  }

  #[test]
  fn load_group_parsing_error() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base.yml");
      std::fs::write(&file, "test").unwrap();

      assert!(matches!(
        load_match_group(&file)
          .unwrap_err()
          .downcast::<LoadError>()
          .unwrap(),
        LoadError::ParsingError(_)
      ));
    });
  }

  #[test]
  fn load_group_yaml_format() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base.yml");
      std::fs::write(
        &file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      assert_eq!(load_match_group(&file).unwrap().matches.len(), 1);
    });
  }

  #[test]
  fn load_group_yaml_format_2() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base.yaml");
      std::fs::write(
        &file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      assert_eq!(load_match_group(&file).unwrap().matches.len(), 1);
    });
  }

  #[test]
  fn load_group_yaml_format_casing() {
    use_test_directory(|_, match_dir, _| {
      let file = match_dir.join("base.YML");
      std::fs::write(
        &file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      assert_eq!(load_match_group(&file).unwrap().matches.len(), 1);
    });
  }
}
