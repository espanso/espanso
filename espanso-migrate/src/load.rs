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
use path_slash::PathExt;
use std::{collections::HashMap, path::Path};
use thiserror::Error;
use walkdir::WalkDir;
use yaml_rust::{yaml::Hash, Yaml, YamlLoader};

pub fn load(config_dir: &Path) -> Result<HashMap<String, Hash>> {
  if !config_dir.is_dir() {
    return Err(LoadError::NotDirectory.into());
  }

  let mut input_files = HashMap::new();

  for entry in WalkDir::new(config_dir) {
    match entry {
      Ok(entry) => {
        // Skip directories
        if entry.path().is_dir() {
          continue;
        }

        // Skip non-yaml files
        let extension = entry
          .path()
          .extension()
          .map(|s| s.to_string_lossy().to_ascii_lowercase())
          .unwrap_or_default();

        if extension != "yaml" && extension != "yml" {
          continue;
        }

        match entry.path().strip_prefix(config_dir) {
          Ok(relative_path) => {
            let corrected_path = relative_path.to_slash_lossy();

            if corrected_path.is_empty() {
              continue;
            }

            match std::fs::read_to_string(entry.path()) {
              Ok(content) => {
                // Empty files are not valid YAML, but we want to handle them anyway
                if content.trim().is_empty() {
                  input_files.insert(corrected_path, Hash::new());
                } else {
                  match YamlLoader::load_from_str(&content) {
                    Ok(mut yaml) => {
                      if !yaml.is_empty() {
                        let yaml = yaml.remove(0);
                        if let Yaml::Hash(hash) = yaml {
                          input_files.insert(corrected_path, hash);
                        } else {
                          eprintln!(
                            "yaml file does not have a valid format: {}",
                            entry.path().display()
                          );
                        }
                      } else {
                        eprintln!(
                          "error, found empty document while reading entry: {}",
                          entry.path().display()
                        );
                      }
                    }
                    Err(err) => {
                      eprintln!(
                        "experienced error while parsing file: {}, error: {}",
                        entry.path().display(),
                        err
                      );
                    }
                  }
                }
              }
              Err(err) => {
                eprintln!(
                  "error while reading entry: {}, error: {}",
                  entry.path().display(),
                  err
                );
              }
            }
          }
          Err(err) => {
            eprintln!(
              "error while analyzing entry: {}, error: {}",
              entry.path().display(),
              err
            );
          }
        }
      }
      Err(err) => {
        eprintln!("experienced error while reading entry: {err}");
      }
    }
  }

  Ok(input_files)
}

#[derive(Error, Debug)]
pub enum LoadError {
  #[error("the provided legacy_config_dir is not a directory")]
  NotDirectory,
}
