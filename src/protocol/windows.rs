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

use super::IPCCommand;
use log::info;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;

use crate::config::ConfigSet;
use crate::event::*;
use crate::protocol::{process_event, send_command};

pub struct WindowsIPCServer {
    config_set: ConfigSet,
    event_channel: Sender<Event>,
}

impl WindowsIPCServer {
    pub fn new(config_set: ConfigSet, event_channel: Sender<Event>) -> WindowsIPCServer {
        WindowsIPCServer {
            config_set,
            event_channel,
        }
    }
}

impl super::IPCServer for WindowsIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        let server_port = self.config_set.default.ipc_server_port;
        std::thread::Builder::new()
            .name("ipc_server".to_string())
            .spawn(move || {
                let listener = TcpListener::bind(format!("127.0.0.1:{}", server_port))
                    .expect("Error binding to IPC server port");

                info!(
                    "Binded to IPC tcp socket: {}",
                    listener.local_addr().unwrap().to_string()
                );

                for stream in listener.incoming() {
                    process_event(&event_channel, stream);
                }
            })
            .expect("Unable to spawn IPC server thread");
    }
}

pub struct WindowsIPCClient {
    config_set: ConfigSet,
}

impl WindowsIPCClient {
    pub fn new(config_set: ConfigSet) -> WindowsIPCClient {
        WindowsIPCClient { config_set }
    }
}

impl super::IPCClient for WindowsIPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String> {
        let stream =
            TcpStream::connect(("127.0.0.1", self.config_set.default.ipc_server_port as u16));

        send_command(command, stream)
    }
}
