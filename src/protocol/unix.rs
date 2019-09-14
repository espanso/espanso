use std::io::{BufRead, BufReader, Read};
use std::io::Write;
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use log::{info, error};
use std::sync::mpsc::Sender;
use super::IPCCommand;

use crate::context;
use crate::context::get_data_dir;
use crate::event::*;

const UNIX_SOCKET_NAME : &str = "espanso.sock";

pub struct UnixIPCServer {
    event_channel: Sender<Event>,
}

impl UnixIPCServer {
    pub fn new(event_channel: Sender<Event>) -> UnixIPCServer {
        UnixIPCServer {event_channel}
    }
}

impl super::IPCServer for UnixIPCServer {
    fn start(&self) {
        let event_channel = self.event_channel.clone();
        std::thread::spawn(move || {
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
                                    event_channel.send(event).expect("Broken event channel");
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

pub struct UnixIPCClient {

}

impl UnixIPCClient {
    pub fn new() -> UnixIPCClient {
        UnixIPCClient{}
    }
}

impl super::IPCClient for UnixIPCClient {
    fn send_command(&self, command: IPCCommand) {
        let espanso_dir = context::get_data_dir();
        let unix_socket = espanso_dir.join(UNIX_SOCKET_NAME);

        // Open the stream
        let mut stream = UnixStream::connect(unix_socket);
        match stream {
            Ok(mut stream) => {
                let json_str = serde_json::to_string(&command);
                if let Ok(json_str) = json_str {
                    stream.write_all(json_str.as_bytes()).unwrap_or_else(|e| {
                        println!("Can't write to IPC socket");
                    });
                }
            },
            Err(e) => {
                println!("Can't connect to daemon: {}", e);
            }
        }
    }
}