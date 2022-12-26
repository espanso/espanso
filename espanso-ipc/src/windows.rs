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

use crate::util::read_line;
use anyhow::Result;
use log::{error, info};
use named_pipe::{ConnectingServer, PipeClient, PipeOptions};
use serde::{de::DeserializeOwned, Serialize};
use std::io::Write;

use crate::{EventHandler, EventHandlerResponse, IPCClient, IPCClientError, IPCServer};

const DEFAULT_CLIENT_TIMEOUT: u32 = 2000;

pub struct WinIPCServer {
  server: Option<ConnectingServer>,
}

impl WinIPCServer {
  pub fn new(id: &str) -> Result<Self> {
    let pipe_name = format!("\\\\.\\pipe\\{}", id);

    let options = PipeOptions::new(&pipe_name);
    let server = Some(options.single()?);

    info!("binded to named pipe: {}", pipe_name);

    Ok(Self { server })
  }
}

impl<Event: Send + Sync + DeserializeOwned + Serialize> IPCServer<Event> for WinIPCServer {
  fn run(mut self, handler: EventHandler<Event>) -> anyhow::Result<()> {
    let server = self
      .server
      .take()
      .expect("unable to extract IPC server handle");
    let mut stream = server.wait()?;

    loop {
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

      stream = stream.disconnect()?.wait()?;
    }
  }
}

pub struct WinIPCClient {
  stream: PipeClient,
}

impl WinIPCClient {
  pub fn new(id: &str) -> Result<Self> {
    let pipe_name = format!("\\\\.\\pipe\\{}", id);

    let stream = PipeClient::connect_ms(pipe_name, DEFAULT_CLIENT_TIMEOUT)?;
    Ok(Self { stream })
  }
}

impl<Event: Serialize + DeserializeOwned> IPCClient<Event> for WinIPCClient {
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
