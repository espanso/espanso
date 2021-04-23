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

use self::ui::util::convert_icon_paths_to_tray_vec;

use super::{CliModule, CliModuleArgs};

mod config;
mod engine;
mod ui;

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: true,
    subcommand: "worker".to_string(),
    entry: worker_main,
    ..Default::default()
  }
}

fn worker_main(args: CliModuleArgs) {
  let config_store = args
    .config_store
    .expect("missing config store in worker main");
  let match_store = args
    .match_store
    .expect("missing match store in worker main");

  let paths = args.paths.expect("missing paths in worker main");

  let icon_paths =
    self::ui::icon::load_icon_paths(&paths.runtime).expect("unable to initialize icons");

  let (remote, mut eventloop) = espanso_ui::create_ui(espanso_ui::UIOptions {
    // TODO: handle show icon
    icon_paths: convert_icon_paths_to_tray_vec(&icon_paths),
    notification_icon_path: icon_paths
      .logo
      .map(|path| path.to_string_lossy().to_string()),
    ..Default::default()
  })
  .expect("unable to create tray icon UI module");

  eventloop
    .initialize()
    .expect("unable to initialize UI module");

  // TODO: pass the remote
  // Initialize the engine on another thread and start it
  engine::initialize_and_spawn(paths.clone(), config_store, match_store)
    .expect("unable to initialize engine");

  eventloop.run(Box::new(move |event| {
    // TODO: handle event
  }));
}
