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

use std::path::Path;

use crate::{
  ipc::{create_ipc_client_to_worker, IPCEvent},
  lock::acquire_worker_lock,
};

use super::{CliModule, CliModuleArgs};
use anyhow::{bail, Result};
use espanso_ipc::IPCClient;

pub fn new() -> CliModule {
  CliModule {
    requires_paths: true,
    subcommand: "cmd".to_string(),
    entry: cmd_main,
    ..Default::default()
  }
}

fn cmd_main(args: CliModuleArgs) -> i32 {
  let cli_args = args.cli_args.expect("missing cli_args");
  let paths = args.paths.expect("missing paths");

  let event = if cli_args.subcommand_matches("enable").is_some() {
    IPCEvent::EnableRequest
  } else if cli_args.subcommand_matches("disable").is_some() {
    IPCEvent::DisableRequest
  } else if cli_args.subcommand_matches("toggle").is_some() {
    IPCEvent::ToggleRequest
  } else if cli_args.subcommand_matches("search").is_some() {
    IPCEvent::OpenSearchBar
  } else if cli_args.subcommand_matches("search").is_some() {
    IPCEvent::OpenConfigFolder
  } else {
    eprintln!("unknown command, please run `espanso cmd --help` to see a list of valid ones.");
    return 1;
  };

  if let Err(error) = send_event_to_worker(&paths.runtime, event) {
    eprintln!("unable to send command, error: {error:?}");
    return 2;
  }

  0
}

fn send_event_to_worker(runtime_path: &Path, event: IPCEvent) -> Result<()> {
  if acquire_worker_lock(runtime_path).is_some() {
    bail!("Worker process is not running, please start Espanso first.")
  }

  let mut client = create_ipc_client_to_worker(runtime_path)?;
  client.send_async(event)
}
