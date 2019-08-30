#[cfg(target_os = "windows")]
mod windows;

use std::sync::mpsc;

pub trait KeyboardBackend {
    fn initialize(&self);
    fn start(&self);
}

#[cfg(target_os = "windows")]
pub fn get_backend(sender: mpsc::Sender<char>) -> impl KeyboardBackend{
    windows::WindowsKeyboardBackend{sender }
}