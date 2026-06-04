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

    let initial_module = args
        .cli_args
        .as_ref()
        .and_then(|m| m.value_of("module"))
        .map(|m| espanso_gui::app::Module::from_str(m))
        .unwrap_or(espanso_gui::app::Module::MatchManager);

    let config_dir = paths.config.clone();
    let runtime_dir = paths.runtime.clone();

    info!(
        "launching espanso management GUI (embedded), config={}",
        config_dir.display()
    );

    // macOS requires winit EventLoop on the main thread.
    // When called from `espanso gui` CLI, we ARE on the main thread.
    espanso_gui::run(Some(config_dir), Some(runtime_dir), initial_module);
    0
}
