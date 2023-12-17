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

use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use espanso_ipc::IPCClient;
use espanso_path::Paths;

use crate::{
  ipc::{create_ipc_client_to_worker, IPCEvent, RequestMatchExpansionPayload},
  lock::acquire_worker_lock,
};

pub fn exec_main(cli_args: &ArgMatches, paths: &Paths) -> Result<()> {
  let trigger = cli_args.value_of("trigger");
  let args = cli_args.values_of("arg");

  if trigger.is_none() || trigger.map(str::is_empty).unwrap_or(false) {
    bail!("You need to specify the --trigger 'trigger' option. Run `espanso match exec --help` for more information.");
  }

  if acquire_worker_lock(&paths.runtime).is_some() {
    bail!("Worker process is not running, please start Espanso first.")
  }

  let mut client = create_ipc_client_to_worker(&paths.runtime)?;

  let mut match_args = HashMap::new();
  if let Some(args) = args {
    args.for_each(|arg| {
      let tokens = arg.split_once('=');
      if let Some((key, value)) = tokens {
        match_args.insert(key.to_string(), value.to_string());
      } else {
        eprintln!("invalid format for argument '{arg}', you should follow the 'name=value' format");
      }
    });
  }

  client
    .send_async(IPCEvent::RequestMatchExpansion(
      RequestMatchExpansionPayload {
        trigger: trigger.map(String::from),
        args: match_args,
      },
    ))
    .context("unable to send payload to worker process")?;

  Ok(())
}
