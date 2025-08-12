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

use crate::exit_code::{ADD_TO_PATH_FAILURE, ADD_TO_PATH_SUCCESS};

use super::{CliModule, CliModuleArgs};
use log::error;

pub fn new() -> CliModule {
    CliModule {
        enable_logs: true,
        disable_logs_terminal_output: true,
        log_mode: super::LogMode::AppendOnly,
        subcommand: "env-path".to_string(),
        entry: env_path_main,
        ..Default::default()
    }
}

fn env_path_main(args: CliModuleArgs) -> i32 {
    let cli_args = args.cli_args.expect("missing cli_args");

    let elevated_priviledge_prompt = cli_args.is_present("prompt");

    if cli_args.subcommand_matches("register").is_some() {
        if let Err(error) = crate::path::add_espanso_to_path(elevated_priviledge_prompt) {
            error_print_and_log(&format!(
                "Unable to add 'espanso' command to PATH: {error:?}"
            ));
            return ADD_TO_PATH_FAILURE;
        }
    } else if cli_args.subcommand_matches("unregister").is_some() {
        if let Err(error) = crate::path::remove_espanso_from_path(elevated_priviledge_prompt) {
            error_print_and_log(&format!(
                "Unable to remove 'espanso' command from PATH: {error:?}"
            ));
            return ADD_TO_PATH_FAILURE;
        }
    } else {
        eprintln!("Please specify a subcommand, either `espanso env-path register` to add the 'espanso' command or `espanso env-path unregister` to remove it");
        return ADD_TO_PATH_FAILURE;
    }

    ADD_TO_PATH_SUCCESS
}

fn error_print_and_log(msg: &str) {
    error!("{msg}");
    eprintln!("{msg}");
}
