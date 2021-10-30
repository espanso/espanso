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

mod list;

pub fn new() -> CliModule {
  CliModule {
    requires_config: true,
    subcommand: "match".to_string(),
    entry: match_main,
    ..Default::default()
  }
}

fn match_main(args: CliModuleArgs) -> i32 {
  let cli_args = args.cli_args.expect("missing cli_args");
  let config_store = args.config_store.expect("missing config_store");
  let match_store = args.match_store.expect("missing match_store");

  if let Some(sub_args) = cli_args.subcommand_matches("list") {
    if let Err(err) = list::list_main(sub_args, config_store, match_store) {
      eprintln!("unable to list matches: {:?}", err);
      return 1;
    }
  } else if let Some(_sub_args) = cli_args.subcommand_matches("exec") {
    todo!();
  } else {
    eprintln!("Invalid use, please run 'espanso match --help' to get more information.");
    return 1;
  }

  0
}
