use std::io::{BufRead, BufReader, Read};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use log::{info, error};
use std::sync::mpsc::Sender;
use super::IPCCommand;

use crate::context;
use crate::context::get_data_dir;
use crate::event::*;

const UNIX_SOCKET_NAME : &str = "espanso.sock";

pub struct UnixIPCManager {
    event_channel: Sender<Event>,
}

impl UnixIPCManager {
    pub fn new(event_channel: Sender<Event>) -> UnixIPCManager {
        UnixIPCManager{event_channel}
    }
}

impl super::IPCManager for UnixIPCManager {
    fn start_server(&self) {
        std::thread::spawn(|| {
            let espanso_dir = context::get_data_dir();
            let unix_socket = espanso_dir.join(UNIX_SOCKET_NAME);

            std::fs::remove_file(unix_socket.clone());
            let listener = UnixListener::bind(unix_socket.clone()).expect("Can't bind to Unix Socket");

            info!("Binded to IPC unix socket: {}", unix_socket.as_path().display());

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut json_str= String::new();
                        let mut buf_reader = BufReader::new(stream);
                        buf_reader.read_to_string(&mut json_str);

                        let command : Result<IPCCommand, serde_json::Error> = serde_json::from_str(&json_str);
                        match command {
                            Ok(command) => {
                                let event = command.to_event();
                                if let Some(event) = event {
                                    // TODO: send event to event channel
                                }
                            },
                            Err(e) => {
                                error!("Error deserializing JSON command: {}", e);
                            },
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        break;
                    }
                }
            }
        });
    }
}