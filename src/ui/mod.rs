#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait UIManager {
    fn initialize(&self);
    fn notify(&self, message: &str);
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_uimanager() -> impl UIManager {
    let manager = macos::MacUIManager{};
    manager.initialize();
    manager
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_uimanager() -> impl UIManager {
    let manager = linux::LinuxUIManager{};
    manager.initialize();
    manager
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_uimanager() -> impl UIManager {
    let manager = windows::WindowsUIManager{};
    manager.initialize();
    manager
}