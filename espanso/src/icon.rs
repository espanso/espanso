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

const ICON_BINARY: &[u8] = include_bytes!("res/icon.png");
const LOGO_NO_BACKGROUND_BINARY: &[u8] = include_bytes!("res/logo_no_background.png");

#[cfg(target_os = "windows")]
const WINDOWS_NORMAL_DARK_ICO_BINARY: &[u8] = include_bytes!("res/windows/normal_dark.ico");
#[cfg(target_os = "windows")]
const WINDOWS_DISABLED_DARK_ICO_BINARY: &[u8] = include_bytes!("res/windows/disabled_dark.ico");
#[cfg(target_os = "windows")]
const WINDOWS_LOGO_ICO_BINARY: &[u8] = include_bytes!("res/windows/logo.ico");


#[cfg(target_os = "macos")]
const MAC_BINARY: &[u8] = include_bytes!("res/macos/icon.png");
#[cfg(target_os = "macos")]
const MAC_DISABLED_BINARY: &[u8] = include_bytes!("res/macos/icondisabled.png");
#[cfg(target_os = "macos")]
const MAC_SYSTEM_DISABLED_BINARY: &[u8] = include_bytes!("res/macos/iconsystemdisabled.png");
#[cfg(target_os = "macos")]
const MAC_ACCESSIBILITY_1_BINARY: &[u8] = include_bytes!("res/accessibility_1.png");
#[cfg(target_os = "macos")]
const MAC_ACCESSIBILITY_2_BINARY: &[u8] = include_bytes!("res/accessibility_2.png");

#[derive(Debug, Default)]
pub struct IconPaths {
  pub form_icon: Option<PathBuf>,
  pub search_icon: Option<PathBuf>,
  pub wizard_icon: Option<PathBuf>,

  pub tray_icon_normal: Option<PathBuf>,
  pub tray_icon_disabled: Option<PathBuf>,
  pub tray_icon_system_disabled: Option<PathBuf>,

  pub accessibility_image_1: Option<PathBuf>,
  pub accessibility_image_2: Option<PathBuf>,

  pub logo: Option<PathBuf>, 
  pub logo_no_background: Option<PathBuf>,
}

#[cfg(target_os = "windows")]
pub fn load_icon_paths(runtime_dir: &Path) -> Result<IconPaths> {
  Ok(IconPaths {
    form_icon: Some(extract_icon(WINDOWS_LOGO_ICO_BINARY, &runtime_dir.join("form.ico"))?),
    search_icon: Some(extract_icon(ICON_BINARY, &runtime_dir.join("search.png"))?),
    wizard_icon: Some(extract_icon(WINDOWS_LOGO_ICO_BINARY, &runtime_dir.join("wizard.ico"))?),
    tray_icon_normal: Some(extract_icon(WINDOWS_NORMAL_DARK_ICO_BINARY, &runtime_dir.join("normal.ico"))?),
    tray_icon_disabled: Some(extract_icon(WINDOWS_DISABLED_DARK_ICO_BINARY, &runtime_dir.join("disabled.ico"))?),
    logo: Some(extract_icon(ICON_BINARY, &runtime_dir.join("icon.png"))?),
    logo_no_background: Some(extract_icon(LOGO_NO_BACKGROUND_BINARY, &runtime_dir.join("icon_no_background.png"))?),
    ..Default::default()
  })
}

#[cfg(target_os = "macos")]
pub fn load_icon_paths(runtime_dir: &Path) -> Result<IconPaths> {
  Ok(IconPaths {
    search_icon: Some(extract_icon(ICON_BINARY, &runtime_dir.join("search.png"))?),
    tray_icon_normal: Some(extract_icon(MAC_BINARY, &runtime_dir.join("normal.png"))?),
    tray_icon_disabled: Some(extract_icon(MAC_DISABLED_BINARY, &runtime_dir.join("disabled.png"))?),
    tray_icon_system_disabled: Some(extract_icon(MAC_SYSTEM_DISABLED_BINARY, &runtime_dir.join("systemdisabled.png"))?),
    logo: Some(extract_icon(ICON_BINARY, &runtime_dir.join("icon.png"))?),
    logo_no_background: Some(extract_icon(LOGO_NO_BACKGROUND_BINARY, &runtime_dir.join("icon_no_background.png"))?),
    accessibility_image_1: Some(extract_icon(MAC_ACCESSIBILITY_1_BINARY, &runtime_dir.join("accessibility_1.png"))?),
    accessibility_image_2: Some(extract_icon(MAC_ACCESSIBILITY_2_BINARY, &runtime_dir.join("accessibility_2.png"))?),
    ..Default::default()
  })
}

#[cfg(target_os = "linux")]
pub fn load_icon_paths(runtime_dir: &Path) -> Result<IconPaths> {
  Ok(IconPaths {
    logo: Some(extract_icon(ICON_BINARY, &runtime_dir.join("icon.png"))?),
    search_icon: Some(extract_icon(ICON_BINARY, &runtime_dir.join("search.png"))?),
    logo_no_background: Some(extract_icon(LOGO_NO_BACKGROUND_BINARY, &runtime_dir.join("icon_no_background.png"))?),
    ..Default::default()
  })
}

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
