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

#[allow(unused_imports)]
#[macro_use]
#[cfg(test)]
extern crate include_dir;

#[allow(unused_imports)]
#[macro_use]
#[cfg(test)]
extern crate test_case;

use std::path::Path;

use anyhow::Result;
use fs_extra::dir::CopyOptions;
use tempdir::TempDir;
use thiserror::Error;

mod convert;
mod load;
mod render;

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

// TODO: test also with non-lowercase file names

pub fn migrate(config_dir: &Path, packages_dir: &Path, output_dir: &Path) -> Result<()> {
  if !config_dir.is_dir() {
    return Err(MigrationError::InvalidConfigDir.into());
  }

  let working_dir = TempDir::new("espanso-migration")?;

  fs_extra::dir::copy(
    config_dir,
    working_dir.path(),
    &CopyOptions {
      content_only: true,
      ..Default::default()
    },
  )?;

  // If packages are located within the config_dir, no need to copy them in
  // the working directory
  if packages_dir.parent() != Some(config_dir) {
    fs_extra::dir::copy(packages_dir, working_dir.path(), &CopyOptions::new())?;
  }

  // Create the output directory
  if output_dir.exists() {
    return Err(MigrationError::OutputDirAlreadyPresent.into());
  }

  std::fs::create_dir_all(output_dir)?;

  // Convert the configurations
  let legacy_files = load::load(working_dir.path())?;
  let converted_files = convert::convert(legacy_files);
  let rendered_files = render::render(converted_files)?;

  for (file, content) in rendered_files {
    let target = output_dir.join(file);

    if let Some(parent) = target.parent() {
      if !parent.is_dir() {
        std::fs::create_dir_all(parent)?;
      }
    }

    std::fs::write(target, content)?;
  }

  // Copy all non-YAML directories
  for entry in std::fs::read_dir(working_dir.path())? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      let dir_name = path.file_name();
      if let Some(name) = dir_name.map(|s| s.to_string_lossy().to_string().to_lowercase()) {
        if name != "user" && name != "packages" {
          fs_extra::dir::copy(path, output_dir, &CopyOptions::new())?;
        }
      }
    }
  }

  Ok(())
}

#[derive(Error, Debug)]
pub enum MigrationError {
  #[error("invalid config directory")]
  InvalidConfigDir,

  #[error("output directory already present")]
  OutputDirAlreadyPresent,
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, fs::create_dir_all, path::Path};

  use super::*;
  use include_dir::{include_dir, Dir};
  use tempdir::TempDir;
  use test_case::test_case;

  use pretty_assertions::assert_eq as assert_peq;
  use yaml_rust::{yaml::Hash, Yaml};

  fn run_with_temp_dir(test_data: &Dir, action: impl FnOnce(&Path, &Path)) {
    let tmp_dir = TempDir::new("espanso-migrate").unwrap();
    let tmp_path = tmp_dir.path();
    let legacy_path = tmp_dir.path().join("legacy");
    let expected_path = tmp_dir.path().join("expected");

    for entry in test_data.find("**/*").unwrap() {
      let entry_path = entry.path();

      let entry_path_str = entry_path.to_string_lossy().to_string();
      if entry_path_str.is_empty() {
        continue;
      }

      let target = tmp_path.join(entry_path);

      if entry_path.extension().is_none() {
        create_dir_all(target).unwrap();
      } else {
        std::fs::write(target, test_data.get_file(entry_path).unwrap().contents()).unwrap();
      }
    }

    action(&legacy_path, &expected_path);
  }

  fn to_sorted_list<T>(hash: HashMap<String, T>) -> Vec<(String, T)> {
    let mut tuples: Vec<(String, T)> = hash.into_iter().collect();
    tuples.sort_by_key(|(k, _)| k.clone());
    tuples
  }

  fn to_sorted_hash(hash: &Hash) -> Vec<(String, &Yaml)> {
    let mut tuples: Vec<(String, &Yaml)> = hash
      .into_iter()
      .map(|(k, v)| (k.as_str().unwrap().to_string(), v))
      .collect();
    tuples.sort_by_key(|(k, _)| k.clone());
    tuples
  }

  fn list_files_in_dir(dir: &Path) -> Vec<String> {
    let prefix = dir.to_string_lossy().to_string();
    fs_extra::dir::get_dir_content(dir)
      .unwrap()
      .files
      .into_iter()
      .map(|file| file.trim_start_matches(&prefix).to_string())
      .collect()
  }

  static SIMPLE_CASE: Dir = include_dir!("test/simple");
  static BASE_CASE: Dir = include_dir!("test/base");
  static ALL_PARAMS_CASE: Dir = include_dir!("test/all_params");
  static OTHER_DIRS_CASE: Dir = include_dir!("test/other_dirs");
  static FORM_SYNTAX: Dir = include_dir!("test/form_syntax");

  #[allow(clippy::unused_unit)]
  #[test_case(&SIMPLE_CASE; "simple case")]
  #[test_case(&BASE_CASE; "base case")]
  #[test_case(&ALL_PARAMS_CASE; "all config parameters case")]
  #[test_case(&OTHER_DIRS_CASE; "other directories case")]
  #[test_case(&FORM_SYNTAX; "form syntax")]
  fn test_migration(test_data: &Dir) {
    run_with_temp_dir(test_data, |legacy, expected| {
      let tmp_out_dir = TempDir::new("espanso-migrate-out").unwrap();
      let output_dir = tmp_out_dir.path().join("out");

      migrate(legacy, &legacy.join("packages"), &output_dir).unwrap();

      let converted_files = load::load(&output_dir).unwrap();

      // Verify configuration content
      let expected_files = load::load(expected).unwrap();
      assert_eq!(converted_files.len(), expected_files.len());
      for (file, converted) in to_sorted_list(converted_files) {
        assert_peq!(
          to_sorted_hash(&converted),
          to_sorted_hash(expected_files.get(&file).unwrap())
        );
      }

      // Verify file structure
      assert_peq!(list_files_in_dir(expected), list_files_in_dir(&output_dir));
    });
  }
}
