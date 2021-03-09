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
use crossbeam::channel::{unbounded, Receiver};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use thiserror::Error;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(target_os = "windows"))]
pub mod unix;

pub trait IPCServer<Event> {
  fn run(&self) -> Result<()>;
  fn accept_one(&self) -> Result<()>;
}

pub trait IPCClient<Event> {
  fn send(&self, event: Event) -> Result<()>;
}

#[cfg(not(target_os = "windows"))]
pub fn server<Event: Send + Sync + DeserializeOwned>(
  id: &str,
  parent_dir: &Path,
) -> Result<(impl IPCServer<Event>, Receiver<Event>)> {
  let (sender, receiver) = unbounded();
  let server = unix::UnixIPCServer::new(id, parent_dir, sender)?;
  Ok((server, receiver))
}

#[cfg(not(target_os = "windows"))]
pub fn client<Event: Serialize>(id: &str, parent_dir: &Path) -> Result<impl IPCClient<Event>> {
  let client = unix::UnixIPCClient::new(id, parent_dir)?;
  Ok(client)
}

#[cfg(target_os = "windows")]
pub fn server<Event: Send + Sync + DeserializeOwned>(
  id: &str,
  _: &Path,
) -> Result<(impl IPCServer<Event>, Receiver<Event>)> {
  let (sender, receiver) = unbounded();
  let server = windows::WinIPCServer::new(id, sender)?;
  Ok((server, receiver))
}

#[cfg(target_os = "windows")]
pub fn client<Event: Serialize>(id: &str, _: &Path) -> Result<impl IPCClient<Event>> {
  let client = windows::WinIPCClient::new(id)?;
  Ok(client)
}

#[derive(Error, Debug)]
pub enum IPCServerError {
  #[error("stream ended")]
  StreamEnded(#[from] std::io::Error),

  #[error("send failed")]
  SendFailed(),
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize)]
  enum Event {
    Bar,
    Foo(String),
  }

  #[test]
  fn ipc_works_correctly() {
    let (server, receiver) = server::<Event>("testespansoipc", &std::env::temp_dir()).unwrap();
    let server_handle = std::thread::spawn(move || {
      server.accept_one().unwrap();
    });

    // TODO: avoid delay and change the IPC code so that we can wait for the IPC
    //std::thread::sleep(std::time::Duration::from_secs(1));

    let client = client::<Event>("testespansoipc", &std::env::temp_dir()).unwrap();
    client.send(Event::Foo("hello".to_string())).unwrap();

    let event = receiver.recv().unwrap();
    assert!(matches!(event, Event::Foo(x) if x == "hello"));

    server_handle.join().unwrap();
  }

  #[test]
  fn ipc_client_fails_to_send() {
    let client = client::<Event>("testespansoipc", &std::env::temp_dir()).unwrap();
    assert!(client.send(Event::Foo("hello".to_string())).is_err());
  }
}
