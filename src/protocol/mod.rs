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

// UNIX IMPLEMENTATION
#[cfg(not(target_os = "windows"))]
pub fn get_ipc_server(event_channel: Sender<Event>) -> impl IPCServer {
    unix::UnixIPCServer::new(event_channel)
}

#[cfg(not(target_os = "windows"))]
pub fn get_ipc_client() -> impl IPCClient {
    unix::UnixIPCClient::new()
}