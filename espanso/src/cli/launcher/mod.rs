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

use log::{error};
use self::util::MigrationError;
use crate::preferences::Preferences;
use crate::exit_code::{LAUNCHER_CONFIG_DIR_POPULATION_FAILURE, LAUNCHER_SUCCESS};

use super::{CliModule, CliModuleArgs};

mod accessibility;
mod daemon;
mod util;

// TODO: test also with modulo feature disabled

pub fn new() -> CliModule {
  #[allow(clippy::needless_update)]
  CliModule {
    requires_paths: true,
    enable_logs: false,
    subcommand: "launcher".to_string(),
    show_in_dock: true,
    entry: launcher_main,
    ..Default::default()
  }
}

#[cfg(feature = "modulo")]
fn launcher_main(args: CliModuleArgs) -> i32 {
  use espanso_modulo::wizard::{MigrationResult, WizardHandlers, WizardOptions};

  // TODO: should we create a non-gui wizard? We can also use it for the non-modulo versions of espanso

  let paths = args.paths.expect("missing paths in launcher main");
  let paths_overrides  = args.paths_overrides.expect("missing paths overrides in launcher main");
  let icon_paths = crate::icon::load_icon_paths(&paths.runtime).expect("unable to load icon paths");

  let preferences =
    crate::preferences::get_default(&paths.runtime).expect("unable to initialize preferences");

  let is_welcome_page_enabled = !preferences.has_completed_wizard();

  let is_move_bundle_page_enabled = false; // TODO

  let is_legacy_version_page_enabled = util::is_legacy_version_running(&paths.runtime);
  let runtime_dir_clone = paths.runtime.clone();
  let is_legacy_version_running_handler =
    Box::new(move || util::is_legacy_version_running(&runtime_dir_clone));

  let is_migrate_page_enabled = espanso_config::is_legacy_config(&paths.config);
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

  let is_add_path_page_enabled =
    if cfg!(not(target_os = "linux")) && !preferences.has_completed_wizard() {
      if cfg!(target_os = "macos") {
        !crate::path::is_espanso_in_path()
      } else {
        if paths.is_portable_mode {
          false
        } else {
          !crate::path::is_espanso_in_path()
        }
      }
    } else {
      false
    };
  // TODO: consider also Windows case?
  let add_to_path_handler = Box::new(move || match util::add_espanso_to_path() {
    Ok(_) => true,
    Err(error) => {
      eprintln!("Add to path returned error: {}", error);
      false
    }
  });

  let is_accessibility_page_enabled = if cfg!(target_os = "macos") {
    !accessibility::is_accessibility_enabled()
  } else {
    false
  };
  let is_accessibility_enabled_handler = Box::new(move || {
    accessibility::is_accessibility_enabled()
  });
  let enable_accessibility_handler = Box::new(move || {
    accessibility::prompt_enable_accessibility();
  });

  let preferences_clone = preferences.clone();
  let on_completed_handler = Box::new(move || {
    preferences_clone.set_completed_wizard(true);
  });

  // TODO: show a "espanso is now running page at the end" (it should be triggered everytime
  // espanso is started, unless the user unchecks "show this message at startup")
  // This page could also be used when the user starts espanso, but an instance is already running.

  // Only show the wizard if a panel should be displayed
  let should_launch_daemon = if is_welcome_page_enabled
    || is_move_bundle_page_enabled
    || is_legacy_version_page_enabled
    || is_migrate_page_enabled
    || is_add_path_page_enabled
    || is_accessibility_page_enabled
  {
    let successful = espanso_modulo::wizard::show(WizardOptions {
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
      accessibility_image_1_path: icon_paths
        .accessibility_image_1
        .map(|path| path.to_string_lossy().to_string()),
      accessibility_image_2_path: icon_paths
        .accessibility_image_2
        .map(|path| path.to_string_lossy().to_string()),
      handlers: WizardHandlers {
        is_legacy_version_running: Some(is_legacy_version_running_handler),
        backup_and_migrate: Some(backup_and_migrate_handler),
        add_to_path: Some(add_to_path_handler),
        enable_accessibility: Some(enable_accessibility_handler),
        is_accessibility_enabled: Some(is_accessibility_enabled_handler),
        on_completed: Some(on_completed_handler),
      },
    });

    successful
  } else {
    true
  };

  if !espanso_config::is_legacy_config(&paths.config) {
    if let Err(err) = crate::config::populate_default_config(&paths.config) {
      error!("Error populating the config directory: {:?}", err);

      // TODO: show an error message with GUI
      return LAUNCHER_CONFIG_DIR_POPULATION_FAILURE;
    }
  }

  if should_launch_daemon {
    // We hide the dock icon on macOS to avoid having it around when the daemon is running
    #[cfg(target_os = "macos")]
    {
      espanso_mac_utils::convert_to_background_app();
    }

    daemon::launch_daemon(&paths_overrides).expect("failed to launch daemon");
  }

  LAUNCHER_SUCCESS
}

#[cfg(not(feature = "modulo"))]
fn launcher_main(_: CliModuleArgs) -> i32 {
  // TODO: handle what happens here

  0
}
