#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait SystemManager {
    fn initialize(&self);
    fn get_current_window_title(&self) -> Option<String>;
    fn get_current_window_class(&self) -> Option<String>;
    fn get_current_window_executable(&self) -> Option<String>;
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_manager() -> impl SystemManager {
    let manager = linux::LinuxSystemManager{};
    manager.initialize();
    manager
}