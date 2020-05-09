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

use std::os::unix::net::{UnixStream,UnixListener};
use log::{info, warn};
use std::sync::mpsc::Sender;
use super::IPCCommand;

use crate::context;
use crate::event::*;
use crate::protocol::{process_event, send_command};
use super::Service;

const DAEMON_UNIX_SOCKET_NAME : &str = "espanso.sock";
const WORKER_UNIX_SOCKET_NAME : &str = "worker.sock";

pub struct UnixIPCServer {
    service: Service,
    event_channel: Sender<Event>,
}

impl UnixIPCServer {
    pub fn new(service: Service, event_channel: Sender<Event>) -> UnixIPCServer {
        UnixIPCServer {
            service,
            event_channel
        }
    }
}

fn get_unix_name(service: &Service) -> String{
    match service {
        Service::Daemon => {DAEMON_UNIX_SOCKET_NAME.to_owned()},
        Service::Worker => {WORKER_UNIX_SOCKET_NAME.to_owned()},
    }
}

impl super::IPCServer for UnixIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        let socket_name = get_unix_name(&self.service);
        std::thread::Builder::new().name("ipc_server".to_string()).spawn(move || {
            let espanso_dir = context::get_data_dir();
            let unix_socket = espanso_dir.join(socket_name);

            std::fs::remove_file(unix_socket.clone()).unwrap_or_else(|e| {
                warn!("Unable to delete Unix socket: {}", e);
            });
            let listener = UnixListener::bind(unix_socket.clone()).expect("Can't bind to Unix Socket");

            info!("Binded to IPC unix socket: {}", unix_socket.as_path().display());

            for stream in listener.incoming() {
                process_event(&event_channel, stream);
            }
        }).expect("Unable to spawn IPC server thread");
    }
}

pub struct UnixIPCClient {
    service: Service,
}

impl UnixIPCClient {
    pub fn new(service: Service) -> UnixIPCClient {
        UnixIPCClient{service}
    }
}

impl super::IPCClient for UnixIPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String> {
        let espanso_dir = context::get_data_dir();
        let socket_name = get_unix_name(&self.service);
        let unix_socket = espanso_dir.join(socket_name);

        // Open the stream
        let stream = UnixStream::connect(unix_socket);

        send_command(command, stream)
    }
}