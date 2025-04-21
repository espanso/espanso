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

pub fn new() -> CliModule {
  CliModule {
    requires_paths: true,
    requires_config: true,
    subcommand: "path".to_string(),
    entry: path_main,
    ..Default::default()
  }
}

fn path_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths argument");
  let cli_args = args.cli_args.expect("missing cli_args argument");

  if cli_args.subcommand_matches("config").is_some() {
    println!("{}", paths.config.to_string_lossy());
  } else if cli_args.subcommand_matches("packages").is_some() {
    println!("{}", paths.packages.to_string_lossy());
  } else if cli_args.subcommand_matches("data").is_some()
    || cli_args.subcommand_matches("runtime").is_some()
  {
    println!("{}", paths.runtime.to_string_lossy());
  } else if cli_args.subcommand_matches("default").is_some() {
    println!(
      "{}",
      paths
        .config
        .join("config")
        .join("default.yml")
        .to_string_lossy()
    );
  } else if cli_args.subcommand_matches("base").is_some() {
    println!(
      "{}",
      paths
        .config
        .join("match")
        .join("base.yml")
        .to_string_lossy()
    );
  } else {
    println!("Config: {}", paths.config.to_string_lossy());
    println!("Packages: {}", paths.packages.to_string_lossy());
    println!("Runtime: {}", paths.runtime.to_string_lossy());
  }

  0
}
