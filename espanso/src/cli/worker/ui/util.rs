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

use espanso_ui::icons::TrayIcon;

use crate::icon::IconPaths;

// TODO: test
pub fn convert_icon_paths_to_tray_vec(icon_paths: &IconPaths) -> Vec<(TrayIcon, String)> {
  let mut paths = Vec::new();

  if let Some(normal) = &icon_paths.tray_icon_normal {
    paths.push((TrayIcon::Normal, normal.to_string_lossy().to_string()));
  }

  if let Some(disabled) = &icon_paths.tray_icon_disabled {
    paths.push((TrayIcon::Disabled, disabled.to_string_lossy().to_string()));
  }

  if let Some(system_disabled) = &icon_paths.tray_icon_system_disabled {
    paths.push((TrayIcon::SystemDisabled, system_disabled.to_string_lossy().to_string()));
  }

  paths
}