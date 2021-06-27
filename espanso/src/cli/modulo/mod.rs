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

#[cfg(feature = "modulo")]
mod form;
#[cfg(feature = "modulo")]
mod search;
#[cfg(feature = "modulo")]
mod welcome;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    enable_logs: false,
    subcommand: "modulo".to_string(),
    entry: modulo_main,
    ..Default::default()
  }
}

#[cfg(feature = "modulo")]
fn modulo_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths in modulo main");
  let cli_args = args.cli_args.expect("missing cli_args in modulo main");
  let icon_paths = crate::icon::load_icon_paths(&paths.runtime).expect("unable to load icon paths");

  if let Some(matches) = cli_args.subcommand_matches("form") {
    return form::form_main(matches, &icon_paths);
  }

  if let Some(matches) = cli_args.subcommand_matches("search") {
    return search::search_main(matches, &icon_paths);
  }

  if let Some(_) = cli_args.subcommand_matches("welcome") {
    return welcome::welcome_main(&paths, &icon_paths);
  }

  0
}

#[cfg(not(feature = "modulo"))]
fn modulo_main(_: CliModuleArgs) -> i32 {
  panic!("this version of espanso was not compiled with 'modulo' support, please obtain a version that does or recompile it with the 'modulo' feature flag");
}
