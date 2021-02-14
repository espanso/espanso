use icons::TrayIcon;
use anyhow::Result;

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
  fn initialize(&mut self);
  fn run(&self, event_callback: UIEventCallback);
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
pub fn create_ui(_options: UIOptions) -> Result<Box<dyn Injector>> {
  // TODO: refactor
  Ok(Box::new(win32::Win32Injector::new()))
}

#[cfg(target_os = "macos")]
pub fn create_ui(_options: UIOptions) -> Result<Box<dyn Injector>> {
  // TODO: refactor
  Ok(Box::new(mac::MacInjector::new()))
}

#[cfg(target_os = "linux")]
pub fn create_ui(options: UIOptions) -> Result<(Box<dyn UIRemote>, Box<dyn UIEventLoop>)> {
  // TODO: here we could avoid panicking and instead return a good result
  let (remote, eventloop) = linux::create(linux::LinuxUIOptions {
    notification_icon_path: options.notification_icon_path.expect("missing notification icon path")
  });
  Ok((Box::new(remote), Box::new(eventloop)))
}