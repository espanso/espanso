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

#[derive(Debug)]
pub enum KeyModifier {
    CTRL,
    SHIFT,
    ALT,
    META,
    BACKSPACE,
}

#[derive(Debug)]
pub enum KeyEvent {
    Char(char),
    Modifier(KeyModifier)
}

pub trait KeyboardSender {
    fn send_string(&self, s: &str);
    fn delete_string(&self, count: i32);
}

// WINDOWS IMPLEMENTATIONS

#[cfg(target_os = "windows")]
pub fn get_interceptor(sender: mpsc::Sender<KeyEvent>) -> impl KeyboardInterceptor {
    windows::WindowsKeyboardInterceptor {sender}
}

#[cfg(target_os = "windows")]
pub fn get_sender() -> impl KeyboardSender {
    windows::WindowsKeyboardSender{}
}

// LINUX IMPLEMENTATIONS

#[cfg(target_os = "linux")]
pub fn get_interceptor(sender: mpsc::Sender<KeyEvent>) -> impl KeyboardInterceptor {
    linux::LinuxKeyboardInterceptor {sender}
}

#[cfg(target_os = "linux")]
pub fn get_sender() -> impl KeyboardSender {
    linux::LinuxKeyboardSender{}
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_interceptor(sender: mpsc::Sender<KeyEvent>) -> impl KeyboardInterceptor {
    macos::MacKeyboardInterceptor {sender}
}

#[cfg(target_os = "macos")]
pub fn get_sender() -> impl KeyboardSender {
    macos::MacKeyboardSender{}
}