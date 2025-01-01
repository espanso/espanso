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

use anyhow::{Context, Result};
use espanso_config::{
  config::ConfigStore,
  error::{ErrorLevel, NonFatalErrorSet},
  matches::store::MatchStore,
};
use log::{error, info, warn};
use std::path::Path;

const DEFAULT_CONFIG_FILE_CONTENT: &str = include_str!("./res/config/default.yml");
const DEFAULT_MATCH_FILE_CONTENT: &str = include_str!("./res/config/base.yml");

pub fn populate_default_config(config_dir: &Path) -> Result<()> {
  if !config_dir.is_dir() {
    info!(
      "generating base configuration directory in: {:?}",
      config_dir
    );
    std::fs::create_dir_all(config_dir)?;
  }

  let sub_config_dir = config_dir.join("config");
  let sub_match_dir = config_dir.join("match");

  if !sub_config_dir.is_dir() {
    info!("generating config directory in: {:?}", sub_config_dir);
    std::fs::create_dir_all(&sub_config_dir)?;
  }
  if !sub_match_dir.is_dir() {
    info!("generating match directory in: {:?}", sub_match_dir);
    std::fs::create_dir_all(&sub_match_dir)?;
  }

  let default_file = sub_config_dir.join("default.yml");
  let match_file = sub_match_dir.join("base.yml");

  if !default_file.is_file() {
    info!(
      "populating default.yml file with initial content: {:?}",
      default_file
    );
    std::fs::write(default_file, DEFAULT_CONFIG_FILE_CONTENT)?;
  }
  if !match_file.is_file() {
    info!(
      "populating base.yml file with initial content: {:?}",
      match_file
    );
    std::fs::write(match_file, DEFAULT_MATCH_FILE_CONTENT)?;
  }

  Ok(())
}

pub struct ConfigLoadResult {
  pub config_store: Box<dyn ConfigStore>,
  pub match_store: Box<dyn MatchStore>,
  pub non_fatal_errors: Vec<NonFatalErrorSet>,
}

pub fn load_config(config_path: &Path) -> Result<ConfigLoadResult> {
  let (config_store, match_store, non_fatal_errors) =
    espanso_config::load(config_path).context("unable to load config")?;

  // TODO: add an option to avoid dumping the errors in the logs
  if !non_fatal_errors.is_empty() {
    warn!("------- detected some errors in the configuration: -------");
    for non_fatal_error_set in &non_fatal_errors {
      warn!(
        ">>> {}",
        non_fatal_error_set.file.to_string_lossy().to_string()
      );
      for record in &non_fatal_error_set.errors {
        if record.level == ErrorLevel::Error {
          error!("{:?}", record.error);
        } else {
          warn!("{:?}", record.error);
        }
      }
    }
    warn!("-----------------------------------------------------------");
  }

  Ok(ConfigLoadResult {
    // Apply the built-in patches
    config_store: crate::patch::patch_store(config_store),
    match_store,
    non_fatal_errors,
  })
}
