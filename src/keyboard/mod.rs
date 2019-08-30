#[cfg(target_os = "windows")]
mod windows;

use std::sync::mpsc;

pub trait KeyboardInterceptor {
    fn initialize(&self);
    fn start(&self);
}

pub trait KeyboardSender {
    fn send_string(&self, s: &str);
}

#[cfg(target_os = "windows")]
pub fn get_backend(sender: mpsc::Sender<char>) -> (impl KeyboardInterceptor, impl KeyboardSender) {
    (windows::WindowsKeyboardInterceptor {sender}, windows::WindowsKeyboardSender{})
}