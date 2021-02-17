use icons::TrayIcon;
use anyhow::Result;
use thiserror::Error;

pub mod event;
pub mod icons;
pub mod menu;

#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod mac;

pub trait UIRemote {
  fn update_tray_icon(&self, icon: TrayIcon);
  fn show_notification(&self, message: &str);
  fn show_context_menu(&self, menu: &menu::Menu);
}

pub type UIEventCallback = Box<dyn Fn(event::UIEvent)>;
pub trait UIEventLoop {
  fn initialize(&mut self) -> Result<()>;
  fn run(&self, event_callback: UIEventCallback) -> Result<()>;
}

pub struct UIOptions {
  pub show_icon: bool,
  pub icon_paths: Vec<(TrayIcon, String)>,
  pub notification_icon_path: Option<String>,
}

impl Default for UIOptions {
  fn default() -> Self {
    Self {
      show_icon: true,
      icon_paths: Vec::new(),
      notification_icon_path: None,
    }
  }
}

#[cfg(target_os = "windows")]
pub fn create_ui(options: UIOptions) -> Result<(Box<dyn UIRemote>, Box<dyn UIEventLoop>)> {
  let (remote, eventloop) = win32::create(win32::Win32UIOptions {
    show_icon: options.show_icon,
    icon_paths: &options.icon_paths,
    notification_icon_path: options.notification_icon_path.ok_or_else(|| UIError::MissingOption("notification icon".to_string()))?,
  })?;
  Ok((Box::new(remote), Box::new(eventloop)))
}

#[cfg(target_os = "macos")]
pub fn create_ui(options: UIOptions) -> Result<(Box<dyn UIRemote>, Box<dyn UIEventLoop>)> {
  let (remote, eventloop) = mac::create(mac::MacUIOptions {
    show_icon: options.show_icon,
    icon_paths: &options.icon_paths,
  })?;
  Ok((Box::new(remote), Box::new(eventloop)))
}

#[cfg(target_os = "linux")]
pub fn create_ui(options: UIOptions) -> Result<(Box<dyn UIRemote>, Box<dyn UIEventLoop>)> {
  let (remote, eventloop) = linux::create(linux::LinuxUIOptions {
    notification_icon_path: options.notification_icon_path.ok_or(UIError::MissingOption("notification icon".to_string()))?,
  });
  Ok((Box::new(remote), Box::new(eventloop)))
}

#[derive(Error, Debug)]
pub enum UIError {
  #[error("missing required option for ui: `{0}`")]
  MissingOption(String),
}