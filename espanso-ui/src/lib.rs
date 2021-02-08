pub mod event;
pub mod icons;
pub mod menu;

#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod mac;