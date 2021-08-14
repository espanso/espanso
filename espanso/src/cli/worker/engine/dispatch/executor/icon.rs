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

use espanso_ui::{UIRemote, icons::TrayIcon};

use espanso_engine::{dispatch::IconHandler, event::ui::IconStatus};

pub struct IconHandlerAdapter<'a> {
  remote: &'a dyn UIRemote,
}

impl<'a> IconHandlerAdapter<'a> {
  pub fn new(remote: &'a dyn UIRemote) -> Self {
    Self { remote }
  }
}

impl<'a> IconHandler for IconHandlerAdapter<'a> {
  fn update_icon(&self, status: &IconStatus) -> anyhow::Result<()> {
    let icon = match status {
      IconStatus::Enabled => TrayIcon::Normal,
      IconStatus::Disabled => TrayIcon::Disabled, 
      IconStatus::SecureInputDisabled => TrayIcon::SystemDisabled, 
    };

    self.remote.update_tray_icon(icon);

    Ok(())
  }
}