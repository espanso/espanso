#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait ClipboardManager {
    fn initialize(&self);
    fn get_clipboard(&self) -> Option<String>;
    fn set_clipboard(&self, payload: &str);
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_manager() -> impl ClipboardManager {
    let manager = linux::LinuxClipboardManager{};
    manager.initialize();
    manager
}