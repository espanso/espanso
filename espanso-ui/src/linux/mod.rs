use log::error;
use notify_rust::Notification;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct LinuxUIOptions {
  pub notification_icon_path: String,
}

pub fn create(options: LinuxUIOptions) -> (LinuxRemote, LinuxEventLoop) {
  let (tx, rx) = mpsc::channel();
  let remote = LinuxRemote::new(tx, options.notification_icon_path);
  let eventloop = LinuxEventLoop::new(rx);
  (remote, eventloop)
}

pub struct LinuxRemote {
  tx: Sender<()>,
  notification_icon_path: String,
}

impl LinuxRemote {
  pub fn new(tx: Sender<()>, notification_icon_path: String) -> Self {
    Self {
      tx,
      notification_icon_path,
    }
  }

  pub fn stop(&self) -> anyhow::Result<()> {
    Ok(self.tx.send(())?)
  }

  pub fn show_notification(&self, message: &str) {
    if let Err(error) = Notification::new()
      .summary("Espanso")
      .body(message)
      .icon(&self.notification_icon_path)
      .show()
    {
      error!("Unable to show notification: {}", error);
    }
  }
}

pub struct LinuxEventLoop {
  rx: Receiver<()>,
}

impl LinuxEventLoop {
  pub fn new(rx: Receiver<()>) -> Self {
    Self { rx }
  }

  pub fn initialize(&self) {
    // NOOP on linux
  }

  pub fn run(&self) {
    // We don't run an event loop on Linux as there is no tray icon or application window needed.
    // Thad said, we still need a way to block this method, and thus we use a channel
    self.rx.recv().expect("Unable to block the LinuxEventLoop");
  }
}
