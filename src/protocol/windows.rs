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

use crate::config::Configs;
use crate::event::*;
use crate::protocol::{process_event, send_command, Service};
use named_pipe::{PipeOptions, PipeServer, PipeClient};
use crate::context;
use std::io::Error;
use std::path::PathBuf;

const DAEMON_WIN_PIPE_NAME: &str = "\\\\.\\pipe\\espansodaemon";
const WORKER_WIN_PIPE_NAME: &str = "\\\\.\\pipe\\espansoworker";
const CLIENT_TIMEOUT: u32 = 2000;

pub struct WindowsIPCServer {
    service: Service,
    event_channel: Sender<Event>,
}

fn get_pipe_name(service: &Service) -> String {
    match service {
        Service::Daemon => DAEMON_WIN_PIPE_NAME.to_owned(),
        Service::Worker => WORKER_WIN_PIPE_NAME.to_owned(),
    }
}


impl WindowsIPCServer {
    pub fn new(
        service: Service,
        event_channel: Sender<Event>,
    ) -> WindowsIPCServer {
        WindowsIPCServer {
            service,
            event_channel,
        }
    }
}

impl super::IPCServer for WindowsIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        let pipe_name = get_pipe_name(&self.service);
        std::thread::Builder::new()
            .name("ipc_server".to_string())
            .spawn(move || {
                let options = PipeOptions::new(&pipe_name);

                info!(
                    "Binding to named pipe: {}",
                    pipe_name
                );

                loop {
                    let server = options.single().expect("unable to initialize IPC named pipe");
                    let pipe_server = server.wait();
                    process_event(&event_channel, pipe_server);
                }
            })
            .expect("Unable to spawn IPC server thread");
    }
}

pub struct WindowsIPCClient {
    service: Service,
}

impl WindowsIPCClient {
    pub fn new(service: Service) -> WindowsIPCClient {
        WindowsIPCClient { service }
    }
}

impl super::IPCClient for WindowsIPCClient {
    fn send_command(&self, command: IPCCommand) -> Result<(), String> {
        let pipe_name = get_pipe_name(&self.service);
        let client = PipeClient::connect_ms(pipe_name, CLIENT_TIMEOUT);

        send_command(command, client)
    }
}
