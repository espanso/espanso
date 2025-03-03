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

use crate::{
  error_eprintln,
  exit_code::{
    configure_custom_panic_hook, PACKAGE_INSTALL_FAILED, PACKAGE_LIST_FAILED, PACKAGE_SUCCESS,
    PACKAGE_UNEXPECTED_FAILURE, PACKAGE_UNINSTALL_FAILED, PACKAGE_UPDATE_FAILED,
    PACKAGE_UPDATE_PARTIAL_FAILURE,
  },
};

use super::{CliModule, CliModuleArgs};

mod install;
mod list;
mod uninstall;
mod update;

pub fn new() -> CliModule {
  CliModule {
    enable_logs: true,
    disable_logs_terminal_output: true,
    requires_paths: true,
    subcommand: "package".to_string(),
    log_mode: super::LogMode::AppendOnly,
    entry: package_main,
    ..Default::default()
  }
}

fn package_main(args: CliModuleArgs) -> i32 {
  configure_custom_panic_hook(PACKAGE_UNEXPECTED_FAILURE);

  let paths = args.paths.expect("missing paths argument");
  let cli_args = args.cli_args.expect("missing cli_args");

  if let Some(sub_matches) = cli_args.subcommand_matches("install") {
    if let Err(err) = install::install_package(&paths, sub_matches) {
      error_eprintln!("unable to install package: {:?}", err);
      return PACKAGE_INSTALL_FAILED;
    }
  } else if let Some(sub_matches) = cli_args.subcommand_matches("uninstall") {
    if let Err(err) = uninstall::uninstall_package(&paths, sub_matches) {
      error_eprintln!("unable to uninstall package: {:?}", err);
      return PACKAGE_UNINSTALL_FAILED;
    }
  } else if let Some(sub_matches) = cli_args.subcommand_matches("list") {
    if let Err(err) = list::list_packages(&paths, sub_matches) {
      error_eprintln!("unable to list packages: {:?}", err);
      return PACKAGE_LIST_FAILED;
    }
  } else if let Some(sub_matches) = cli_args.subcommand_matches("update") {
    match update::update_package(&paths, sub_matches) {
      Ok(update::UpdateResults::PartialFailure) => {
        error_eprintln!("some packages were updated, but not all of them. Check the previous log for more information");
        return PACKAGE_UPDATE_PARTIAL_FAILURE;
      }
      Err(err) => {
        error_eprintln!("unable to update package: {:?}", err);
        return PACKAGE_UPDATE_FAILED;
      }
      _ => {}
    }
  }

  PACKAGE_SUCCESS
}
