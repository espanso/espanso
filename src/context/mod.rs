#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub(crate) mod macos;

use std::sync::mpsc::Sender;
use crate::event::Event;
use std::path::PathBuf;
use std::fs::create_dir_all;

pub trait Context {
    fn eventloop(&self);
}

pub fn get_data_dir() -> PathBuf {
    let data_dir = dirs::data_dir().expect("Can't obtain data_dir(), terminating.");
    let espanso_dir = data_dir.join("espanso");
    create_dir_all(&espanso_dir).expect("Error creating espanso data directory");
    espanso_dir
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    macos::MacContext::new(send_channel)
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    linux::LinuxContext::new(send_channel)
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    windows::WindowsContext::new(send_channel)
}