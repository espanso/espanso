/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

pub use crate::sys::wizard::show;

pub struct WizardOptions {
  pub version: String,

  pub is_welcome_page_enabled: bool,
  pub is_move_bundle_page_enabled: bool,
  pub is_legacy_version_page_enabled: bool,
  pub is_wrong_edition_page_enabled: bool,
  pub is_migrate_page_enabled: bool,
  pub is_auto_start_page_enabled: bool,
  pub is_add_path_page_enabled: bool,
  pub is_accessibility_page_enabled: bool,

  pub window_icon_path: Option<String>,
  pub welcome_image_path: Option<String>,
  pub accessibility_image_1_path: Option<String>,
  pub accessibility_image_2_path: Option<String>,
  pub detected_os: DetectedOS,

  pub handlers: WizardHandlers,
}

pub struct WizardHandlers {
  pub is_legacy_version_running: Option<Box<dyn Fn() -> bool + Send>>,
  pub backup_and_migrate: Option<Box<dyn Fn() -> MigrationResult + Send>>,
  pub auto_start: Option<Box<dyn Fn(bool) -> bool + Send>>,
  pub add_to_path: Option<Box<dyn Fn() -> bool + Send>>,
  pub enable_accessibility: Option<Box<dyn Fn() + Send>>,
  pub is_accessibility_enabled: Option<Box<dyn Fn() -> bool + Send>>,
  pub on_completed: Option<Box<dyn Fn() + Send>>,
}

#[derive(Debug)]
pub enum MigrationResult {
  Success,
  CleanFailure,
  DirtyFailure,
  UnknownFailure,
}

#[derive(Debug)]
pub enum DetectedOS {
  Unknown,
  X11,
  Wayland,
}
