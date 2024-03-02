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

use espanso_config::config::Config;
use espanso_ui::UIRemote;

pub struct NotificationManager<'a> {
  ui_remote: &'a dyn UIRemote,
  config: &'a dyn Config,
}

impl<'a> NotificationManager<'a> {
  pub fn new(ui_remote: &'a dyn UIRemote, config: &'a dyn Config) -> Self {
    NotificationManager { ui_remote, config }
  }

  fn notify(&self, text: &str) {
    if self.config.show_notifications() {
      self.ui_remote.show_notification(text);
    }
  }

  pub fn notify_start(&self) {
    self.notify("Espanso is running!");
  }

  pub fn notify_config_reloaded(&self, is_manual_restart: bool) {
    if is_manual_restart {
      self.notify("Configuration reloaded!");
    } else {
      self.notify(
        "Configuration reloaded! Espanso automatically loads new changes as soon as you save them.",
      );
    }
  }

  pub fn notify_keyboard_layout_reloaded(&self) {
    self.notify("Updated keyboard layout!");
  }
}

impl<'a> espanso_engine::process::NotificationManager for NotificationManager<'a> {
  fn notify_status_change(&self, enabled: bool) {
    // Don't notify the status change outside Linux for now
    if !cfg!(target_os = "linux") {
      return;
    }

    if enabled {
      self.notify("Espanso enabled!");
    } else {
      self.notify("Espanso disabled!");
    }
  }

  fn notify_rendering_error(&self) {
    self
      .notify("An error occurred during rendering, please examine the logs for more information.");
  }
}
