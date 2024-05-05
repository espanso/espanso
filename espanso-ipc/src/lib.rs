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
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use thiserror::Error;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(target_os = "windows"))]
pub mod unix;

mod util;

pub type EventHandler<Event> = Box<dyn Fn(Event) -> EventHandlerResponse<Event>>;

pub enum EventHandlerResponse<Event> {
  NoResponse,
  Response(Event),
  Error(anyhow::Error),
  Exit,
}

pub trait IPCServer<Event> {
  fn run(self, handler: EventHandler<Event>) -> Result<()>;
}

pub trait IPCClient<Event> {
  fn send_sync(&mut self, event: Event) -> Result<Event>;
  fn send_async(&mut self, event: Event) -> Result<()>;
}

#[cfg(not(target_os = "windows"))]
pub fn server<Event: Send + Sync + DeserializeOwned + Serialize>(
  id: &str,
  parent_dir: &Path,
) -> Result<impl IPCServer<Event>> {
  let server = unix::UnixIPCServer::new(id, parent_dir)?;
  Ok(server)
}

#[cfg(not(target_os = "windows"))]
pub fn client<Event: Serialize + DeserializeOwned>(
  id: &str,
  parent_dir: &Path,
) -> Result<impl IPCClient<Event>> {
  let client = unix::UnixIPCClient::new(id, parent_dir)?;
  Ok(client)
}

#[cfg(target_os = "windows")]
pub fn server<Event: Send + Sync + DeserializeOwned + Serialize>(
  id: &str,
  _: &Path,
) -> Result<impl IPCServer<Event>> {
  let server = windows::WinIPCServer::new(id)?;
  Ok(server)
}

#[cfg(target_os = "windows")]
pub fn client<Event: Serialize + DeserializeOwned>(
  id: &str,
  _: &Path,
) -> Result<impl IPCClient<Event>> {
  let client = windows::WinIPCClient::new(id)?;
  Ok(client)
}

#[derive(Error, Debug)]
pub enum IPCServerError {
  #[error("stream ended")]
  StreamEnded,

  #[error("handler reported error `{0}`")]
  HandlerError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum IPCClientError {
  #[error("empty response")]
  EmptyResponse,

  #[error("malformed response received `{0}`")]
  MalformedResponse(#[from] anyhow::Error),

  #[error("message response timed out")]
  Timeout,
}

#[cfg(test)]
mod tests {
  use std::sync::mpsc::channel;

  use super::*;
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize)]
  enum Event {
    Async,
    Sync(String),
    SyncResult(String),
    ExitRequest,
  }

  #[test]
  fn ipc_async_message() {
    let server = server::<Event>("testespansoipcasync", &std::env::temp_dir()).unwrap();

    let client_handle = std::thread::spawn(move || {
      let mut client = client::<Event>("testespansoipcasync", &std::env::temp_dir()).unwrap();

      client.send_async(Event::Async).unwrap();
      client.send_async(Event::ExitRequest).unwrap();
    });

    server
      .run(Box::new(move |event| match event {
        Event::ExitRequest => EventHandlerResponse::Exit,
        evt => {
          assert!(matches!(evt, Event::Async));
          EventHandlerResponse::NoResponse
        }
      }))
      .unwrap();

    client_handle.join().unwrap();
  }

  #[test]
  fn ipc_sync_message() {
    let server = server::<Event>("testespansoipcsync", &std::env::temp_dir()).unwrap();

    let client_handle = std::thread::spawn(move || {
      let mut client = client::<Event>("testespansoipcsync", &std::env::temp_dir()).unwrap();

      let response = client.send_sync(Event::Sync("test".to_owned())).unwrap();
      client.send_async(Event::ExitRequest).unwrap();

      assert!(matches!(response, Event::SyncResult(s) if s == "test"));
    });

    server
      .run(Box::new(move |event| match event {
        Event::ExitRequest => EventHandlerResponse::Exit,
        Event::Sync(s) => EventHandlerResponse::Response(Event::SyncResult(s)),
        _ => EventHandlerResponse::NoResponse,
      }))
      .unwrap();

    client_handle.join().unwrap();
  }

  #[test]
  fn ipc_multiple_sync_with_delay_message() {
    let server = server::<Event>("testespansoipcmultiplesync", &std::env::temp_dir()).unwrap();

    let client_handle = std::thread::spawn(move || {
      let mut client =
        client::<Event>("testespansoipcmultiplesync", &std::env::temp_dir()).unwrap();

      let response = client.send_sync(Event::Sync("test".to_owned())).unwrap();

      std::thread::sleep(std::time::Duration::from_millis(500));

      let response2 = client.send_sync(Event::Sync("test2".to_owned())).unwrap();
      client.send_async(Event::ExitRequest).unwrap();

      assert!(matches!(response, Event::SyncResult(s) if s == "test"));
      assert!(matches!(response2, Event::SyncResult(s) if s == "test2"));
    });

    server
      .run(Box::new(move |event| match event {
        Event::ExitRequest => EventHandlerResponse::Exit,
        Event::Sync(s) => EventHandlerResponse::Response(Event::SyncResult(s)),
        _ => EventHandlerResponse::NoResponse,
      }))
      .unwrap();

    client_handle.join().unwrap();
  }

  #[ignore = "takes too loong to test this (way over 60 seconds)"]
  #[test]
  fn ipc_multiple_clients() {
    let server = server::<Event>("testespansoipcmultiple", &std::env::temp_dir()).unwrap();

    let (tx, rx) = channel();

    let client_handle = std::thread::spawn(move || {
      let mut client = client::<Event>("testespansoipcmultiple", &std::env::temp_dir()).unwrap();

      let response = client.send_sync(Event::Sync("client1".to_owned())).unwrap();

      tx.send(()).unwrap();

      assert!(matches!(response, Event::SyncResult(s) if s == "client1"));
    });

    let client_handle2 = std::thread::spawn(move || {
      let mut client = client::<Event>("testespansoipcmultiple", &std::env::temp_dir()).unwrap();

      let response = client.send_sync(Event::Sync("client2".to_owned())).unwrap();

      // Wait for the other client before terminating
      rx.recv().unwrap();

      client.send_async(Event::ExitRequest).unwrap();

      assert!(matches!(response, Event::SyncResult(s) if s == "client2"));
    });

    server
      .run(Box::new(move |event| match event {
        Event::ExitRequest => EventHandlerResponse::Exit,
        Event::Sync(s) => EventHandlerResponse::Response(Event::SyncResult(s)),
        _ => EventHandlerResponse::NoResponse,
      }))
      .unwrap();

    client_handle.join().unwrap();
    client_handle2.join().unwrap();
  }

  #[test]
  fn ipc_sync_big_payload_message() {
    let server = server::<Event>("testespansoipcsyncbig", &std::env::temp_dir()).unwrap();

    let client_handle = std::thread::spawn(move || {
      let mut client = client::<Event>("testespansoipcsyncbig", &std::env::temp_dir()).unwrap();

      let mut payload = String::new();
      for _ in 0..10000 {
        payload.push_str("log string repeated");
      }
      let response = client.send_sync(Event::Sync(payload.clone())).unwrap();
      client.send_async(Event::ExitRequest).unwrap();

      assert!(matches!(response, Event::SyncResult(s) if s == payload));
    });

    server
      .run(Box::new(move |event| match event {
        Event::ExitRequest => EventHandlerResponse::Exit,
        Event::Sync(s) => EventHandlerResponse::Response(Event::SyncResult(s)),
        _ => EventHandlerResponse::NoResponse,
      }))
      .unwrap();

    client_handle.join().unwrap();
  }
}
