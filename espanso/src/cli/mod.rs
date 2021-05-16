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

use clap::ArgMatches;
use espanso_config::{config::ConfigStore, matches::store::MatchStore};
use espanso_path::Paths;

pub mod daemon;
pub mod log;
pub mod path;
pub mod worker;

pub struct CliModule {
  pub enable_logs: bool,
  pub log_mode: LogMode,
  pub requires_paths: bool,
  pub requires_config: bool,
  pub subcommand: String,
  pub entry: fn(CliModuleArgs),
}

impl Default for CliModule {
  fn default() -> Self {
    Self {
      enable_logs: false,
      log_mode: LogMode::Read,
      requires_paths: false, 
      requires_config: false, 
      subcommand: "".to_string(), 
      entry: |_| {},
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum LogMode {
  Read,
  AppendOnly,
  CleanAndAppend,
}

pub struct CliModuleArgs {
  pub config_store: Option<Box<dyn ConfigStore>>,
  pub match_store: Option<Box<dyn MatchStore>>,
  pub is_legacy_config: bool,
  pub paths: Option<Paths>,
  pub cli_args: Option<ArgMatches<'static>>,
}

impl Default for CliModuleArgs {
  fn default() -> Self {
    Self {
      config_store: None,
      match_store: None,
      is_legacy_config: false,
      paths: None,
      cli_args: None,
    }
  }
}
