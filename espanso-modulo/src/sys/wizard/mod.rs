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

use std::os::raw::{c_char, c_int};
use std::{ffi::CString, sync::Mutex};

use crate::sys::util::convert_to_cstring_or_null;
use crate::{sys::interop::{WIZARD_MIGRATE_RESULT_CLEAN_FAILURE, WIZARD_MIGRATE_RESULT_DIRTY_FAILURE, WIZARD_MIGRATE_RESULT_SUCCESS, WIZARD_MIGRATE_RESULT_UNKNOWN_FAILURE, WizardMetadata}, wizard::{WizardHandlers, WizardOptions}};

lazy_static! {
  static ref HANDLERS: Mutex<Option<WizardHandlers>> = Mutex::new(None);
}

pub fn show(options: WizardOptions) {
  let c_version = CString::new(options.version).expect("unable to convert version to CString");

  let (_c_window_icon_path, c_window_icon_path_ptr) =
    convert_to_cstring_or_null(options.window_icon_path);
  let (_c_welcome_image, c_welcome_image_path_ptr) =
    convert_to_cstring_or_null(options.welcome_image_path);
  let (_c_accessibility_image_1_path, c_accessibility_image_1_path_ptr) =
    convert_to_cstring_or_null(options.accessibility_image_1_path);
  let (_c_accessibility_image_2_path, c_accessibility_image_2_path_ptr) =
    convert_to_cstring_or_null(options.accessibility_image_2_path);

  extern "C" fn is_legacy_version_running() -> c_int {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in is_legacy_version_running method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.is_legacy_version_running.as_ref() {
      if (*handler_ref)() {
        1
      } else {
        0
      }
    } else {
      -1
    }
  }

  extern "C" fn backup_and_migrate() -> c_int {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in backup_and_migrate method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.backup_and_migrate.as_ref() {
      match (*handler_ref)() {
        crate::wizard::MigrationResult::Success => WIZARD_MIGRATE_RESULT_SUCCESS,
        crate::wizard::MigrationResult::CleanFailure => WIZARD_MIGRATE_RESULT_CLEAN_FAILURE,
        crate::wizard::MigrationResult::DirtyFailure => WIZARD_MIGRATE_RESULT_DIRTY_FAILURE,
        crate::wizard::MigrationResult::UnknownFailure => WIZARD_MIGRATE_RESULT_UNKNOWN_FAILURE,
      }
    } else {
      WIZARD_MIGRATE_RESULT_UNKNOWN_FAILURE
    }
  }

  extern "C" fn add_to_path() -> c_int {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in add_to_path method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.add_to_path.as_ref() {
      if (*handler_ref)() {
        1
      } else {
        0
      }
    } else {
      -1
    }
  }

  extern "C" fn enable_accessibility() -> c_int {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in enable_accessibility method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.enable_accessibility.as_ref() {
      (*handler_ref)();
      1
    } else {
      -1
    }
  }

  extern "C" fn is_accessibility_enabled() -> c_int {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in is_accessibility_enabled method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.is_accessibility_enabled.as_ref() {
      if (*handler_ref)() {
        1
      } else {
        0
      }
    } else {
      -1
    }
  }

  extern "C" fn on_completed() {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in on_completed method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.on_completed.as_ref() {
      (*handler_ref)()
    }
  }
  
  {
    let mut lock = HANDLERS.lock().expect("unable to acquire handlers lock");
    *lock = Some(options.handlers)
  }

  let wizard_metadata = WizardMetadata {
    version: c_version.as_ptr(),

    is_welcome_page_enabled: if options.is_welcome_page_enabled {
      1
    } else {
      0
    },
    is_move_bundle_page_enabled: if options.is_move_bundle_page_enabled {
      1
    } else {
      0
    },
    is_legacy_version_page_enabled: if options.is_legacy_version_page_enabled {
      1
    } else {
      0
    },
    is_migrate_page_enabled: if options.is_migrate_page_enabled {
      1
    } else {
      0
    },
    is_add_path_page_enabled: if options.is_add_path_page_enabled {
      1
    } else {
      0
    },
    is_accessibility_page_enabled: if options.is_accessibility_page_enabled {
      1
    } else {
      0
    },

    window_icon_path: c_window_icon_path_ptr,
    welcome_image_path: c_welcome_image_path_ptr,
    accessibility_image_1_path: c_accessibility_image_1_path_ptr,
    accessibility_image_2_path: c_accessibility_image_2_path_ptr,

    is_legacy_version_running,
    backup_and_migrate,
    add_to_path,
    enable_accessibility,
    is_accessibility_enabled,
    on_completed,
  };

  unsafe {
    super::interop::interop_show_wizard(&wizard_metadata);
  }
}