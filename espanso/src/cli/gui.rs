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

use log::info;
use std::process::Command;

use super::{CliModule, CliModuleArgs};

pub fn new() -> CliModule {
    CliModule {
        requires_paths: true,
        enable_logs: false,
        subcommand: "gui".to_string(),
        show_in_dock: true,
        entry: gui_main,
        ..Default::default()
    }
}

fn gui_main(args: CliModuleArgs) -> i32 {
    let paths = args.paths.expect("missing paths in gui main");

    // Determine the path to the espanso-gui binary
    // It should be in the same directory as the main espanso binary
    let gui_binary = find_gui_binary();

    let initial_module = args
        .cli_args
        .as_ref()
        .and_then(|m| m.value_of("module"))
        .unwrap_or("match");

    info!(
        "launching espanso management GUI: {} --config_dir={} --runtime_dir={} {}",
        gui_binary.display(),
        paths.config.display(),
        paths.runtime.display(),
        initial_module
    );

    match Command::new(&gui_binary)
        .arg(format!("--config_dir={}", paths.config.display()))
        .arg(format!("--runtime_dir={}", paths.runtime.display()))
        .arg(initial_module)
        .spawn()
    {
        Ok(_child) => {
            info!("GUI process spawned successfully");
            // Don't wait — let the GUI run independently
            0
        }
        Err(e) => {
            // If the GUI binary isn't found, print a helpful message
            eprintln!("Unable to launch espanso GUI: {e}");
            eprintln!(
                "The espanso-gui binary was not found at: {}",
                gui_binary.display()
            );
            eprintln!("Make sure espanso-gui is installed alongside the main espanso binary.");
            1
        }
    }
}

/// Find the espanso-gui binary.
///
/// On all platforms, it's expected to be in the same directory as the
/// main espanso executable.
fn find_gui_binary() -> std::path::PathBuf {
    let current_exe =
        std::env::current_exe().expect("unable to determine current executable path");
    let exe_dir = current_exe
        .parent()
        .expect("unable to determine executable directory");

    #[cfg(target_os = "windows")]
    let gui_name = "espanso-gui.exe";
    #[cfg(not(target_os = "windows"))]
    let gui_name = "espanso-gui";

    exe_dir.join(gui_name)
}
