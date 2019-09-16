/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use crate::event::Event;
use crate::event::ActionType;
use std::io::{BufReader, Read, Write};
use std::error::Error;
use log::error;
use crate::config::ConfigSet;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
mod unix;

pub trait IPCServer {
    fn start(&self);
}

pub trait IPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IPCCommand {
    pub id: String,

    #[serde(default)]
    pub payload: String,
}

impl IPCCommand {
    fn to_event(&self) -> Option<Event> {
        match self.id.as_ref() {
            "exit" => {
                Some(Event::Action(ActionType::Exit))
            },
            "toggle" => {
                Some(Event::Action(ActionType::Toggle))
            },
            "enable" => {
                Some(Event::Action(ActionType::Enable))
            },
            "disable" => {
                Some(Event::Action(ActionType::Disable))
            },
            _ => None
        }
    }
}

fn process_event<R: Read, E: Error>(event_channel: &Sender<Event>, stream: Result<R, E>) {
    match stream {
        Ok(stream) => {
            let mut json_str= String::new();
            let mut buf_reader = BufReader::new(stream);
            let res = buf_reader.read_to_string(&mut json_str);

            if res.is_ok() {
                let command : Result<IPCCommand, serde_json::Error> = serde_json::from_str(&json_str);
                match command {
                    Ok(command) => {
                        let event = command.to_event();
                        if let Some(event) = event {
                            event_channel.send(event).expect("Broken event channel");
                        }
                    },
                    Err(e) => {
                        error!("Error deserializing JSON command: {}", e);
                    },
                }
            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

fn send_command<W: Write, E: Error>(command: IPCCommand, stream: Result<W, E>) -> Result<(), String>{
    match stream {
        Ok(mut stream) => {
            let json_str = serde_json::to_string(&command);
            if let Ok(json_str) = json_str {
                stream.write_all(json_str.as_bytes()).unwrap_or_else(|e| {
                    println!("Can't write to IPC socket: {}", e);
                });
                return Ok(())
            }
        },
        Err(e) => {
            return Err(format!("Can't connect to daemon: {}", e))
        }
    }

    Err("Can't send command".to_owned())
}

// UNIX IMPLEMENTATION
#[cfg(not(target_os = "windows"))]
pub fn get_ipc_server(_: ConfigSet, event_channel: Sender<Event>) -> impl IPCServer {
    unix::UnixIPCServer::new(event_channel)
}

#[cfg(not(target_os = "windows"))]
pub fn get_ipc_client(_: ConfigSet) -> impl IPCClient {
    unix::UnixIPCClient::new()
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_ipc_server(config_set: ConfigSet, event_channel: Sender<Event>) -> impl IPCServer {
    windows::WindowsIPCServer::new(config_set, event_channel)
}

#[cfg(target_os = "windows")]
pub fn get_ipc_client(config_set: ConfigSet) -> impl IPCClient {
    windows::WindowsIPCClient::new(config_set)
}