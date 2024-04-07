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

use crate::{util::read_line, EventHandlerResponse, IPCClientError};
use anyhow::Result;
use log::{error, info};
use serde::{de::DeserializeOwned, Serialize};
use std::{
  io::Write,
  os::unix::net::{UnixListener, UnixStream},
  path::Path,
};

use crate::{EventHandler, IPCClient, IPCServer};

pub struct UnixIPCServer {
  listener: UnixListener,
}

impl UnixIPCServer {
  pub fn new(id: &str, parent_dir: &Path) -> Result<Self> {
    let socket_path = parent_dir.join(format!("{id}.sock"));

    // Remove previous Unix socket
    if socket_path.exists() {
      std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    info!(
      "binded to IPC unix socket: {}",
      socket_path.to_string_lossy()
    );

    Ok(Self { listener })
  }
}

impl<Event: Send + Sync + DeserializeOwned + Serialize> IPCServer<Event> for UnixIPCServer {
  fn run(self, handler: EventHandler<Event>) -> Result<()> {
    loop {
      let (mut stream, _) = self.listener.accept()?;

      // Read multiple commands from the client
      loop {
        match read_line(&mut stream) {
          Ok(Some(line)) => {
            let event: Result<Event, serde_json::Error> = serde_json::from_str(&line);
            match event {
              Ok(event) => match handler(event) {
                EventHandlerResponse::Response(response) => {
                  let mut json_event = serde_json::to_string(&response)?;
                  json_event.push('\n');
                  stream.write_all(json_event.as_bytes())?;
                  stream.flush()?;
                }
                EventHandlerResponse::NoResponse => {
                  // Async event, no need to reply
                }
                EventHandlerResponse::Error(err) => {
                  error!("ipc handler reported an error: {}", err);
                }
                EventHandlerResponse::Exit => {
                  return Ok(());
                }
              },
              Err(error) => {
                error!("received malformed event from ipc stream: {}", error);
                break;
              }
            }
          }
          Ok(None) => {
            // EOF reached
            break;
          }
          Err(error) => {
            error!("error reading ipc stream: {}", error);
            break;
          }
        }
      }
    }
  }
}

pub struct UnixIPCClient {
  stream: UnixStream,
}

impl UnixIPCClient {
  pub fn new(id: &str, parent_dir: &Path) -> Result<Self> {
    let socket_path = parent_dir.join(format!("{id}.sock"));
    let stream = UnixStream::connect(socket_path)?;

    Ok(Self { stream })
  }
}

impl<Event: Serialize + DeserializeOwned> IPCClient<Event> for UnixIPCClient {
  fn send_sync(&mut self, event: Event) -> Result<Event> {
    {
      let mut json_event = serde_json::to_string(&event)?;
      json_event.push('\n');
      self.stream.write_all(json_event.as_bytes())?;
      self.stream.flush()?;
    }

    // Read the response
    if let Some(line) = read_line(&mut self.stream)? {
      let event: Result<Event, serde_json::Error> = serde_json::from_str(&line);
      match event {
        Ok(response) => Ok(response),
        Err(err) => Err(IPCClientError::MalformedResponse(err.into()).into()),
      }
    } else {
      Err(IPCClientError::EmptyResponse.into())
    }
  }

  fn send_async(&mut self, event: Event) -> Result<()> {
    let mut json_event = serde_json::to_string(&event)?;
    json_event.push('\n');
    self.stream.write_all(json_event.as_bytes())?;
    self.stream.flush()?;

    Ok(())
  }
}
