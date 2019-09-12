#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

use std::sync::mpsc::Sender;
use crate::event::Event;
use std::sync::Arc;

pub trait Context {
    fn eventloop(&self);
}

pub enum MenuItemType {
    Button,
    Separator
}

pub struct MenuItem {

}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {  // TODO
    macos::MacUIManager::new()
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> { // TODO
    let manager = linux::LinuxUIManager{};
    manager.initialize();
    manager
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn new(send_channel: Sender<Event>) -> Box<dyn Context> {
    windows::WindowsContext::new(send_channel)
}