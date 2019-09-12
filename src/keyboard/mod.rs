#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait KeyboardSender {  // TODO: rename KeyboardManager
    fn send_string(&self, s: &str);
    fn send_enter(&self);
    fn trigger_paste(&self);
    fn delete_string(&self, count: i32);
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_sender() -> impl KeyboardSender {
    windows::WindowsKeyboardSender{}
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_sender() -> impl KeyboardSender {
    linux::LinuxKeyboardSender{}
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_sender() -> impl KeyboardSender {
    macos::MacKeyboardSender{}
}