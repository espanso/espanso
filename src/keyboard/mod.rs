#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait KeyboardManager {
    fn send_string(&self, s: &str);
    fn send_enter(&self);
    fn trigger_paste(&self);
    fn delete_string(&self, count: i32);
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_manager() -> impl KeyboardManager {
    windows::WindowsKeyboardManager{}
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_manager() -> impl KeyboardManager {
    linux::LinuxKeyboardManager{}
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_manager() -> impl KeyboardManager {
    macos::MacKeyboardManager{}
}