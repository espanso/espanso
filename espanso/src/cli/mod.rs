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

use std::path::PathBuf;

use clap::ArgMatches;
use espanso_config::{config::ConfigStore, error::NonFatalErrorSet, matches::store::MatchStore};
use espanso_path::Paths;

pub mod cmd;
pub mod daemon;
pub mod edit;
pub mod env_path;
pub mod launcher;
pub mod log;
pub mod match_cli;
pub mod modulo;
pub mod package;
pub mod path;
pub mod service;
pub mod util;
pub mod workaround;
pub mod worker;

// we really need these bools until we rewrite the CLI in the Derive API of clap
#[allow(dead_code, clippy::struct_excessive_bools)]
pub struct CliModule {
  pub enable_logs: bool,
  pub disable_logs_terminal_output: bool,
  pub log_mode: LogMode,
  pub requires_paths: bool,
  pub requires_config: bool,
  pub subcommand: String,
  pub show_in_dock: bool,
  pub requires_linux_capabilities: bool,
  pub entry: fn(CliModuleArgs) -> i32,
}

impl Default for CliModule {
  fn default() -> Self {
    Self {
      enable_logs: false,
      log_mode: LogMode::Read,
      disable_logs_terminal_output: false,
      requires_paths: false,
      requires_config: false,
      subcommand: String::new(),
      show_in_dock: false,
      requires_linux_capabilities: false,
      entry: |_| 0,
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogMode {
  Read,
  AppendOnly,
  CleanAndAppend,
}

#[derive(Default)]
pub struct CliModuleArgs {
  pub config_store: Option<Box<dyn ConfigStore>>,
  pub match_store: Option<Box<dyn MatchStore>>,
  pub non_fatal_errors: Vec<NonFatalErrorSet>,
  pub paths: Option<Paths>,
  pub paths_overrides: Option<PathsOverrides>,
  pub cli_args: Option<ArgMatches>,
}

pub struct PathsOverrides {
  pub config: Option<PathBuf>,
  pub runtime: Option<PathBuf>,
  pub packages: Option<PathBuf>,
}

pub struct CliAlias {
  pub subcommand: String,
  pub forward_into: String,
}
