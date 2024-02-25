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

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::sys;
use crate::sys::interop::{ErrorSetMetadata, TroubleshootingMetadata};
use crate::sys::troubleshooting::interop::OwnedErrorSet;
use crate::sys::util::convert_to_cstring_or_null;
use crate::troubleshooting::{TroubleshootingHandlers, TroubleshootingOptions};
use anyhow::Result;
use lazy_static::lazy_static;

lazy_static! {
  static ref HANDLERS: Mutex<Option<TroubleshootingHandlers>> = Mutex::new(None);
}

#[allow(dead_code)]
mod interop {
  use crate::sys;
  use crate::troubleshooting::{ErrorRecord, ErrorSet};

  use super::interop::{ErrorMetadata, ErrorSetMetadata};

  use super::super::interop::*;
  use std::{ffi::CString, os::raw::c_int};

  pub(crate) struct OwnedErrorSet {
    file_path: Option<CString>,
    errors: Vec<OwnedErrorMetadata>,
    pub(crate) interop_errors: Vec<ErrorMetadata>,
  }

  impl OwnedErrorSet {
    pub fn to_error_set_metadata(&self) -> ErrorSetMetadata {
      let file_path_ptr = if let Some(file_path) = self.file_path.as_ref() {
        file_path.as_ptr()
      } else {
        std::ptr::null()
      };

      ErrorSetMetadata {
        file_path: file_path_ptr,
        errors: self.interop_errors.as_ptr(),
        errors_count: self.interop_errors.len() as c_int,
      }
    }
  }

  impl From<&ErrorSet> for OwnedErrorSet {
    fn from(error_set: &ErrorSet) -> Self {
      let file_path = error_set.file.as_ref().map(|file_path| {
        CString::new(file_path.to_string_lossy().to_string())
          .expect("unable to convert file_path to CString")
      });

      let errors: Vec<OwnedErrorMetadata> = error_set.errors.iter().map(Into::into).collect();

      let interop_errors: Vec<ErrorMetadata> = errors
        .iter()
        .map(sys::troubleshooting::interop::OwnedErrorMetadata::to_error_metadata)
        .collect();

      Self {
        file_path,
        errors,
        interop_errors,
      }
    }
  }

  pub(crate) struct OwnedErrorMetadata {
    level: c_int,
    message: CString,
  }

  impl OwnedErrorMetadata {
    fn to_error_metadata(&self) -> ErrorMetadata {
      ErrorMetadata {
        level: self.level,
        message: self.message.as_ptr(),
      }
    }
  }

  impl From<&ErrorRecord> for OwnedErrorMetadata {
    fn from(item: &ErrorRecord) -> Self {
      let message =
        CString::new(item.message.clone()).expect("unable to convert item message to CString");

      Self {
        level: match item.level {
          crate::troubleshooting::ErrorLevel::Error => ERROR_METADATA_LEVEL_ERROR,
          crate::troubleshooting::ErrorLevel::Warning => ERROR_METADATA_LEVEL_WARNING,
        },
        message,
      }
    }
  }
}

pub fn show(options: TroubleshootingOptions) -> Result<()> {
  let (_c_window_icon_path, c_window_icon_path_ptr) =
    convert_to_cstring_or_null(options.window_icon_path);

  let owned_error_sets: Vec<OwnedErrorSet> = options.error_sets.iter().map(Into::into).collect();
  let error_sets: Vec<ErrorSetMetadata> = owned_error_sets
    .iter()
    .map(sys::troubleshooting::interop::OwnedErrorSet::to_error_set_metadata)
    .collect();

  extern "C" fn dont_show_again_changed(dont_show: c_int) {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in dont_show_again_changed method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.dont_show_again_changed.as_ref() {
      let value = dont_show == 1;
      (*handler_ref)(value);
    }
  }

  extern "C" fn open_file(file_path: *const c_char) {
    let lock = HANDLERS
      .lock()
      .expect("unable to acquire lock in open_file method");
    let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
    if let Some(handler_ref) = handlers_ref.open_file.as_ref() {
      let c_string = unsafe { CStr::from_ptr(file_path) };
      let string = c_string.to_string_lossy();
      let path = PathBuf::from(string.to_string());
      (*handler_ref)(&path);
    }
  }

  {
    let mut lock = HANDLERS.lock().expect("unable to acquire handlers lock");
    *lock = Some(options.handlers);
  }

  let troubleshooting_metadata = TroubleshootingMetadata {
    window_icon_path: c_window_icon_path_ptr,
    is_fatal_error: i32::from(options.is_fatal_error),
    error_sets: error_sets.as_ptr(),
    error_sets_count: error_sets.len() as c_int,
    dont_show_again_changed,
    open_file,
  };

  unsafe {
    super::interop::interop_show_troubleshooting(&troubleshooting_metadata);
  }

  Ok(())
}
