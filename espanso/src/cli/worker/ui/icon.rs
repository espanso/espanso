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

use anyhow::Result;
use log::{debug, info};
use std::path::{Path, PathBuf};

const ICON_BINARY: &[u8] = include_bytes!("../../../res/icon.png");

#[cfg(target_os = "windows")]
const WINDOWS_ICO_BINARY: &[u8] = include_bytes!("../../../res/windows/espanso.ico");
#[cfg(target_os = "windows")]
const WINDOWS_RED_ICO_BINARY: &[u8] = include_bytes!("../../../res/windows/espansored.ico");

// TODO: macos

#[derive(Debug, Default)]
pub struct IconPaths {
  pub form_icon: Option<PathBuf>,

  pub tray_icon_normal: Option<PathBuf>,
  pub tray_icon_disabled: Option<PathBuf>,
  pub tray_icon_system_disabled: Option<PathBuf>, // TODO: secure input

  pub logo: Option<PathBuf>, 
}

#[cfg(target_os = "windows")]
pub fn load_icon_paths(runtime_dir: &Path) -> Result<IconPaths> {
  Ok(IconPaths {
    form_icon: Some(extract_icon(WINDOWS_ICO_BINARY, &runtime_dir.join("form.ico"))?),
    tray_icon_normal: Some(extract_icon(WINDOWS_ICO_BINARY, &runtime_dir.join("normal.ico"))?),
    tray_icon_disabled: Some(extract_icon(WINDOWS_RED_ICO_BINARY, &runtime_dir.join("disabled.ico"))?),
    logo: Some(extract_icon(ICON_BINARY, &runtime_dir.join("icon.png"))?),
    ..Default::default()
  })
}

#[cfg(target_os = "linux")]
pub fn load_icon_paths(runtime_dir: &Path) -> Result<IconPaths> {
  Ok(IconPaths {
    logo: Some(extract_icon(ICON_BINARY, &runtime_dir.join("icon.png"))?),
    ..Default::default()
  })
}

// TODO: macos

// TODO: test
fn extract_icon(data: &[u8], target_file: &Path) -> Result<PathBuf> {
  if target_file.exists() {
    debug!(
      "skipping extraction for '{:?}', as it's already present",
      target_file
    );
    Ok(target_file.to_owned())
  } else {
    std::fs::write(target_file, data)?;
    info!("extracted icon to: {:?}", target_file);
    Ok(target_file.to_owned())
  }
}
