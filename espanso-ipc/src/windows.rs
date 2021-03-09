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
use named_pipe::{PipeClient, PipeOptions};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{BufReader, Read, Write};

use crate::{IPCClient, IPCServer, IPCServerError};

const CLIENT_TIMEOUT: u32 = 2000;

pub struct WinIPCServer<Event> {
  options: PipeOptions,
  sender: Sender<Event>,
}

impl<Event> WinIPCServer<Event> {
  pub fn new(id: &str, sender: Sender<Event>) -> Result<Self> {
    let pipe_name = format!("\\\\.\\pipe\\{}", id);

    let options = PipeOptions::new(&pipe_name);

    info!("binded to named pipe: {}", pipe_name);

    Ok(Self { options, sender })
  }
}

impl<Event: Send + Sync + DeserializeOwned> IPCServer<Event> for WinIPCServer<Event> {
  fn run(&self) -> anyhow::Result<()> {
    loop {
      self.accept_one()?;
    }
  }

  fn accept_one(&self) -> Result<()> {
    let server = self.options.single()?;
    let connection = server.wait();

    match connection {
      Ok(stream) => {
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

pub struct WinIPCClient {
  pipe_name: String,
}

impl WinIPCClient {
  pub fn new(id: &str) -> Result<Self> {
    let pipe_name = format!("\\\\.\\pipe\\{}", id);
    Ok(Self { pipe_name })
  }
}

impl<Event: Serialize> IPCClient<Event> for WinIPCClient {
  fn send(&self, event: Event) -> Result<()> {
    let mut stream = PipeClient::connect_ms(&self.pipe_name, CLIENT_TIMEOUT)?;

    let json_event = serde_json::to_string(&event)?;
    stream.write_all(json_event.as_bytes())?;

    Ok(())
  }
}
