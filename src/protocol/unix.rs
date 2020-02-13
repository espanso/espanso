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
use log::{info, warn};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::Sender;

use crate::context;
use crate::event::*;
use crate::protocol::{process_event, send_command};

const UNIX_SOCKET_NAME: &str = "espanso.sock";

pub struct UnixIPCServer {
    event_channel: Sender<Event>,
}

impl UnixIPCServer {
    pub fn new(event_channel: Sender<Event>) -> UnixIPCServer {
        UnixIPCServer { event_channel }
    }
}

impl super::IPCServer for UnixIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        std::thread::Builder::new()
            .name("ipc_server".to_string())
            .spawn(move || {
                let espanso_dir = context::get_data_dir();
                let unix_socket = espanso_dir.join(UNIX_SOCKET_NAME);

                std::fs::remove_file(unix_socket.clone()).unwrap_or_else(|e| {
                    warn!("Unable to delete Unix socket: {}", e);
                });
                let listener =
                    UnixListener::bind(unix_socket.clone()).expect("Can't bind to Unix Socket");

                info!(
                    "Binded to IPC unix socket: {}",
                    unix_socket.as_path().display()
                );

                for stream in listener.incoming() {
                    process_event(&event_channel, stream);
                }
            })
            .expect("Unable to spawn IPC server thread");
    }
}

pub struct UnixIPCClient {}

impl UnixIPCClient {
    pub fn new() -> UnixIPCClient {
        UnixIPCClient {}
    }
}

impl super::IPCClient for UnixIPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String> {
        let espanso_dir = context::get_data_dir();
        let unix_socket = espanso_dir.join(UNIX_SOCKET_NAME);

        // Open the stream
        let stream = UnixStream::connect(unix_socket);

        send_command(command, stream)
    }
}
