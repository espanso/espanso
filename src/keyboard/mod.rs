#[cfg(target_os = "windows")]
mod windows;

use std::sync::mpsc;

pub trait KeyboardInterceptor {
    fn initialize(&self);
    fn start(&self);
}

pub trait KeyboardSender {
    fn send_string(&self, s: &str);
    fn delete_string(&self, count: i32);
}

#[cfg(target_os = "windows")]
pub fn get_interceptor(sender: mpsc::Sender<char>) -> impl KeyboardInterceptor {
    windows::WindowsKeyboardInterceptor {sender}
}

#[cfg(target_os = "windows")]
pub fn get_sender() -> impl KeyboardSender {
    windows::WindowsKeyboardSender{}
}