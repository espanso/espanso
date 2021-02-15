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
use crossbeam::channel::Sender;
use log::{error, info};
use serde::{de::DeserializeOwned, Serialize};
use std::{
  io::{BufReader, Read, Write},
  os::unix::net::{UnixListener, UnixStream},
  path::{Path, PathBuf},
};

use crate::{IPCClient, IPCServer, IPCServerError};

pub struct UnixIPCServer<Event> {
  listener: UnixListener,
  sender: Sender<Event>,
}

impl<Event> UnixIPCServer<Event> {
  pub fn new(id: &str, parent_dir: &Path, sender: Sender<Event>) -> Result<Self> {
    let socket_path = parent_dir.join(format!("{}.sock", id));

    // Remove previous Unix socket
    if socket_path.exists() {
      std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    info!(
      "binded to IPC unix socket: {}",
      socket_path.to_string_lossy()
    );

    Ok(Self { listener, sender })
  }
}

impl<Event: Send + Sync + DeserializeOwned> IPCServer<Event> for UnixIPCServer<Event> {
  fn run(&self) -> anyhow::Result<()> {
    loop {
      self.accept_one()?;
    }
  }

  fn accept_one(&self) -> Result<()> {
    let connection = self.listener.accept();

    match connection {
      Ok((stream, _)) => {
        let mut json_str = String::new();
        let mut buf_reader = BufReader::new(stream);
        let result = buf_reader.read_to_string(&mut json_str);

        match result {
          Ok(_) => {
            let event: Result<Event, serde_json::Error> = serde_json::from_str(&json_str);
            match event {
              Ok(event) => {
                if self.sender.send(event).is_err() {
                  return Err(IPCServerError::SendFailed().into());
                }
              }
              Err(error) => {
                error!("received malformed event from ipc stream: {}", error);
              }
            }
          }
          Err(error) => {
            error!("error reading ipc stream: {}", error);
          }
        }
      }
      Err(err) => {
        return Err(IPCServerError::StreamEnded(err).into());
      }
    };

    Ok(())
  }
}

pub struct UnixIPCClient {
  socket_path: PathBuf,
}

impl UnixIPCClient {
  pub fn new(id: &str, parent_dir: &Path) -> Result<Self> {
    let socket_path = parent_dir.join(format!("{}.sock", id));

    Ok(Self { socket_path })
  }
}

impl<Event: Serialize> IPCClient<Event> for UnixIPCClient {
  fn send(&self, event: Event) -> Result<()> {
    let mut stream = UnixStream::connect(&self.socket_path)?;

    let json_event = serde_json::to_string(&event)?;
    stream.write_all(json_event.as_bytes())?;

    Ok(())
  }
}
