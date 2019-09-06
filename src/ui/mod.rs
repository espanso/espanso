#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait UIManager {
    fn initialize(&self);
    fn notify(&self, message: &str);

    fn new() -> impl UIManager {
        let manager = get_uimanager();
        manager.initialize();
        manager
    }
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
fn get_uimanager() -> impl UIManager {
    macos::MacUIManager{}
}