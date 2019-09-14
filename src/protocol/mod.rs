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

pub trait IPCManager {
    fn start_server(&self);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IPCCommand {
    id: String,
    payload: String,
}

impl IPCCommand {
    fn to_event(&self) -> Option<Event> {
        match self.id.as_ref() {
            "exit" => {
                Some(Event::Action(ActionType::Exit))
            },
            _ => None
        }
    }
}

// UNIX IMPLEMENTATION
#[cfg(not(target_os = "windows"))]
pub fn get_ipc_manager(event_channel: Sender<Event>) -> impl IPCManager {
    unix::UnixIPCManager::new(event_channel)
}