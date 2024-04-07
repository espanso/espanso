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

use anyhow::Result;
use espanso_ipc::{IPCClient, IPCServer};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub enum IPCEvent {
  Exit,
  ExitAllProcesses,

  EnableRequest,
  DisableRequest,
  ToggleRequest,
  OpenSearchBar,
  OpenConfigFolder,

  RequestMatchExpansion(RequestMatchExpansionPayload),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMatchExpansionPayload {
  pub trigger: Option<String>,
  pub args: HashMap<String, String>,
}

pub fn create_daemon_ipc_server(runtime_dir: &Path) -> Result<impl IPCServer<IPCEvent>> {
  create_ipc_server(runtime_dir, "daemonv2")
}

pub fn create_worker_ipc_server(runtime_dir: &Path) -> Result<impl IPCServer<IPCEvent>> {
  create_ipc_server(runtime_dir, "workerv2")
}

pub fn create_ipc_client_to_worker(runtime_dir: &Path) -> Result<impl IPCClient<IPCEvent>> {
  create_ipc_client(runtime_dir, "workerv2")
}

fn create_ipc_server(runtime_dir: &Path, name: &str) -> Result<impl IPCServer<IPCEvent>> {
  espanso_ipc::server(&format!("espanso{name}"), runtime_dir)
}

fn create_ipc_client(runtime_dir: &Path, target_process: &str) -> Result<impl IPCClient<IPCEvent>> {
  let client = espanso_ipc::client(&format!("espanso{target_process}"), runtime_dir)?;
  Ok(client)
}
