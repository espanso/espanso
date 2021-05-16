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
use crossbeam::channel::Receiver;
use espanso_ipc::{IPCServer, IPCClient};
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IPCEvent {
  Exit,
}

pub fn spawn_daemon_ipc_server(runtime_dir: &Path) -> Result<Receiver<IPCEvent>> {
  spawn_ipc_server(runtime_dir, "daemon")
}

pub fn spawn_worker_ipc_server(runtime_dir: &Path) -> Result<Receiver<IPCEvent>> {
  spawn_ipc_server(runtime_dir, "worker")
}

pub fn create_ipc_client_to_worker(runtime_dir: &Path) -> Result<impl IPCClient<IPCEvent>> {
  create_ipc_client(runtime_dir, "worker")
}

fn spawn_ipc_server(
  runtime_dir: &Path,
  name: &str,
) -> Result<Receiver<IPCEvent>> {
  let (server, receiver) = espanso_ipc::server(&format!("espanso{}", name), runtime_dir)?;

  std::thread::Builder::new().name(format!("espanso-ipc-server-{}", name)).spawn(move || {
    server.run().expect("unable to run ipc server");
  })?;

  // TODO: refactor the ipc server to handle a graceful exit?

  Ok(receiver)
}


fn create_ipc_client(runtime_dir: &Path, target_process: &str) -> Result<impl IPCClient<IPCEvent>> {
  let client = espanso_ipc::client(&format!("espanso{}", target_process), runtime_dir)?;
  Ok(client)
}