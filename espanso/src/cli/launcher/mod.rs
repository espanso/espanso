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

use self::util::MigrationError;
use crate::preferences::Preferences;

use super::{CliModule, CliModuleArgs};

mod util;

// TODO: test also with modulo feature disabled

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    requires_config: true,
    enable_logs: false,
    subcommand: "launcher".to_string(),
    entry: launcher_main,
    ..Default::default()
  }
}

#[cfg(feature = "modulo")]
fn launcher_main(args: CliModuleArgs) -> i32 {
  use espanso_modulo::wizard::{MigrationResult, WizardHandlers, WizardOptions};

  // TODO: should we create a non-gui wizard? We can also use it for the non-modulo versions of espanso

  let paths = args.paths.expect("missing paths in launcher main");
  let icon_paths = crate::icon::load_icon_paths(&paths.runtime).expect("unable to load icon paths");

  let preferences =
    crate::preferences::get_default(&paths.runtime).expect("unable to initialize preferences");

  let is_welcome_page_enabled = !preferences.has_completed_wizard();

  let is_move_bundle_page_enabled = false; // TODO

  let is_legacy_version_page_enabled = util::is_legacy_version_running(&paths.runtime);
  let runtime_dir_clone = paths.runtime.clone();
  let is_legacy_version_running_handler =
    Box::new(move || util::is_legacy_version_running(&runtime_dir_clone));

  let is_migrate_page_enabled = args.is_legacy_config;
  let paths_clone = paths.clone();
  let backup_and_migrate_handler =
    Box::new(move || match util::migrate_configuration(&paths_clone) {
      Ok(_) => MigrationResult::Success,
      Err(error) => match error.downcast_ref::<MigrationError>() {
        Some(MigrationError::DirtyError) => MigrationResult::DirtyFailure,
        Some(MigrationError::CleanError) => MigrationResult::CleanFailure,
        _ => MigrationResult::UnknownFailure,
      },
    });

  // TODO: enable "Add to PATH" page only when NOT in portable mode
  // TODO: if the user clicks on "Continue" after unchecking the "ADD to PATH"
  // checkbox, we should remember (with the kvs) the choice and avoid asking again.
  let is_add_path_page_enabled = if cfg!(target_os = "macos") {
    // TODO: add actual check
    // TODO: consider also Windows case
    true
  } else {
    false
  };

  let is_accessibility_page_enabled = if cfg!(target_os = "macos") {
    // TODO: add actual check
    true
  } else {
    false
  };

  // TODO: show a "espanso is now running page at the end" (it should be triggered everytime
  // espanso is started, unless the user unchecks "show this message at startup")
  // This page could also be used when the user starts espanso, but an instance is already running.

  // Only show the wizard if a panel should be displayed
  if is_welcome_page_enabled
    || is_move_bundle_page_enabled
    || is_legacy_version_page_enabled
    || is_migrate_page_enabled
    || is_add_path_page_enabled
    || is_accessibility_page_enabled
  {
    espanso_modulo::wizard::show(WizardOptions {
      version: crate::VERSION.to_string(),
      is_welcome_page_enabled,
      is_move_bundle_page_enabled,
      is_legacy_version_page_enabled,
      is_migrate_page_enabled,
      is_add_path_page_enabled,
      is_accessibility_page_enabled,
      window_icon_path: icon_paths
        .wizard_icon
        .map(|path| path.to_string_lossy().to_string()),
      welcome_image_path: icon_paths
        .logo_no_background
        .map(|path| path.to_string_lossy().to_string()),
      accessibility_image_1_path: None, // TODO
      accessibility_image_2_path: None, // TODO
      handlers: WizardHandlers {
        is_legacy_version_running: Some(is_legacy_version_running_handler),
        backup_and_migrate: Some(backup_and_migrate_handler),
        add_to_path: None,              // TODO
        enable_accessibility: None,     // TODO
        is_accessibility_enabled: None, // TODO
      },
    });

    // TODO: check the wizard return status?
    preferences.set_completed_wizard(true);
  }

  // TODO: initialize config directory if not present

  0
}

#[cfg(not(feature = "modulo"))]
fn launcher_main(_: CliModuleArgs) -> i32 {
  // TODO: handle what happens here

  0
}
