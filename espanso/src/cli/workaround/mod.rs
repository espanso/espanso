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

use super::{CliModule, CliModuleArgs};
use crate::{error_eprintln, exit_code::{WORKAROUND_FAILURE, WORKAROUND_SUCCESS}};

#[cfg(target_os = "macos")]
mod secure_input;

pub fn new() -> CliModule {
  CliModule {
    subcommand: "workaround".to_string(),
    entry: workaround_main,
    ..Default::default()
  }
}

fn workaround_main(args: CliModuleArgs) -> i32 {
  let cli_args = args.cli_args.expect("missing cli_args");

  if cli_args.subcommand_matches("secure-input").is_some() {
    #[cfg(target_os = "macos")]
    {
      if let Err(err) = secure_input::run_secure_input_workaround() {
        error_eprintln!("secure-input workaround reported error: {}", err);
        return WORKAROUND_FAILURE;
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      error_eprintln!("secure-input workaround is only available on macOS");
      return crate::exit_code::WORKAROUND_NOT_AVAILABLE;
    }
  }

  WORKAROUND_SUCCESS
}
