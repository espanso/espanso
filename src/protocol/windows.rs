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

use log::{info};
use std::sync::mpsc::Sender;
use std::net::{TcpListener, TcpStream};
use super::IPCCommand;

use crate::event::*;
use crate::protocol::{process_event, send_command, Service};
use crate::config::{Configs};

pub struct WindowsIPCServer {
    service: Service,
    config: Configs,
    event_channel: Sender<Event>,
}

fn to_port(config: &Configs, service: &Service) -> u16 {
    let port = match service {
        Service::Daemon => {config.ipc_server_port},
        Service::Worker => {config.worker_ipc_server_port},
    };
    port as u16
}

impl WindowsIPCServer {
    pub fn new(service: Service, config: Configs, event_channel: Sender<Event>) -> WindowsIPCServer {
        WindowsIPCServer {service, config, event_channel}
    }
}

impl super::IPCServer for WindowsIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        let server_port = to_port(&self.config, &self.service);
        std::thread::Builder::new().name("ipc_server".to_string()).spawn(move || {
            let listener = TcpListener::bind(
                format!("127.0.0.1:{}", server_port)
                ).expect("Error binding to IPC server port");

            info!("Binded to IPC tcp socket: {}", listener.local_addr().unwrap().to_string());

            for stream in listener.incoming() {
                process_event(&event_channel, stream);
            }
        }).expect("Unable to spawn IPC server thread");
    }
}

pub struct WindowsIPCClient {
    service: Service,
    config: Configs,
}

impl WindowsIPCClient {
    pub fn new(service: Service, config: Configs) -> WindowsIPCClient {
        WindowsIPCClient{service, config}
    }
}

impl super::IPCClient for WindowsIPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String> {
        let port = to_port(&self.config, &self.service);
        let stream = TcpStream::connect(
            ("127.0.0.1", port)
        );

        send_command(command, stream)
    }
}