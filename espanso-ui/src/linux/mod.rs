/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::time::Duration;

use anyhow::Result;
use crossbeam::{
  channel::{unbounded, Receiver, Sender},
  select,
};
use log::error;
use notify_rust::Notification;

use crate::{event::UIEvent, UIEventLoop, UIRemote};

pub struct LinuxUIOptions {
  pub notification_icon_path: String,
}

pub fn create(options: LinuxUIOptions) -> (LinuxRemote, LinuxEventLoop) {
  let (tx, rx) = unbounded();
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
}

impl UIRemote for LinuxRemote {
  fn update_tray_icon(&self, _: crate::icons::TrayIcon) {
    // NOOP on linux
  }

  fn show_notification(&self, message: &str) {
    if let Err(error) = Notification::new()
      .summary("Espanso")
      .body(message)
      .icon(&self.notification_icon_path)
      .show()
    {
      error!("Unable to show notification: {}", error);
    }
  }

  fn show_context_menu(&self, _: &crate::menu::Menu) {
    // NOOP on linux
  }

  fn exit(&self) {
    self
      .stop()
      .expect("unable to send termination signal to ui eventloop");
  }
}

pub struct LinuxEventLoop {
  rx: Receiver<()>,
}

impl LinuxEventLoop {
  pub fn new(rx: Receiver<()>) -> Self {
    Self { rx }
  }
}

impl UIEventLoop for LinuxEventLoop {
  fn initialize(&mut self) -> Result<()> {
    // NOOP on linux
    Ok(())
  }

  fn run(&self, callback: crate::UIEventCallback) -> Result<()> {
    loop {
      select! {
        recv(self.rx) -> result => {
          // We don't run an event loop on Linux as there is no tray icon or application window needed.
          // Thad said, we still need a way to block this method, and thus we use a channel
          match result {
            Ok(()) => {
              // remote.exit() called
              return Ok(());
            }
            Err(error) => {
              error!("Unable to block the LinuxEventLoop: {}", error);
              return Err(error.into());
            }
          }
        },
        default(Duration::from_millis(1000)) => {
          (*callback)(UIEvent::Heartbeat);
        }
      }
    }
  }
}
