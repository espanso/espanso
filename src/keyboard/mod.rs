#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

use std::sync::mpsc;

pub trait KeyboardInterceptor {
    fn initialize(&self);
    fn start(&self);
}

pub trait KeyboardSender {
    fn send_string(&self, s: &str);
    fn delete_string(&self, count: i32);
}

// WINDOWS IMPLEMENTATIONS

#[cfg(target_os = "windows")]
pub fn get_interceptor(sender: mpsc::Sender<char>) -> impl KeyboardInterceptor {
    windows::WindowsKeyboardInterceptor {sender}
}

#[cfg(target_os = "windows")]
pub fn get_sender() -> impl KeyboardSender {
    windows::WindowsKeyboardSender{}
}

// LINUX IMPLEMENTATIONS

#[cfg(target_os = "linux")]
pub fn get_interceptor(sender: mpsc::Sender<char>) -> impl KeyboardInterceptor {
    linux::LinuxKeyboardInterceptor {sender}
}

#[cfg(target_os = "linux")]
pub fn get_sender() -> impl KeyboardSender {
    linux::LinuxKeyboardSender{}
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_interceptor(sender: mpsc::Sender<char>) -> impl KeyboardInterceptor {
    macos::MacKeyboardInterceptor {sender}
}

#[cfg(target_os = "macos")]
pub fn get_sender() -> impl KeyboardSender {
    macos::MacKeyboardSender{}
}