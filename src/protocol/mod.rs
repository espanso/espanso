use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::sync::mpsc::Sender;
use crate::event::Event;
use crate::event::Event::*;
use crate::event::ActionType;
use crate::event::ActionType::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
mod unix;

pub trait IPCServer {
    fn start(&self);
}

pub trait IPCClient {
    fn send_command(&self, command: IPCCommand);
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