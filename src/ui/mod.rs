#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub trait UIManager {
    fn notify(&self, message: &str);
    fn show_menu(&self, menu: Vec<MenuItem>);
}

pub enum MenuItemType {
    Button,
    Separator,
}

pub struct MenuItem {
    pub item_id: i32,
    pub item_type: MenuItemType,
    pub item_name: String,
}

// MAC IMPLEMENTATION
#[cfg(target_os = "macos")]
pub fn get_uimanager() -> impl UIManager {
    macos::MacUIManager::new()
}

// LINUX IMPLEMENTATION
#[cfg(target_os = "linux")]
pub fn get_uimanager() -> impl UIManager {
    linux::LinuxUIManager::new()
}

// WINDOWS IMPLEMENTATION
#[cfg(target_os = "windows")]
pub fn get_uimanager() -> impl UIManager {
    windows::WindowsUIManager::new()
}