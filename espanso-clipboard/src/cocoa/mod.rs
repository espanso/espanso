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

mod ffi;

use std::{
  ffi::{CStr, CString},
  path::PathBuf,
};

use crate::{Clipboard, ClipboardOperationOptions};
use anyhow::Result;
use log::error;
use std::os::raw::c_char;
use thiserror::Error;

pub struct CocoaClipboard {}

impl CocoaClipboard {
  pub fn new() -> Result<Self> {
    Ok(Self {})
  }
}

impl Clipboard for CocoaClipboard {
  fn get_text(&self, _: &ClipboardOperationOptions) -> Option<String> {
    // get the clipbard size
    let length = unsafe { ffi::clipboard_get_length() };
    if length <= 0 {
      return None;
    }

    // allocate the buffer with extra space for null terminator
    let mut buffer: Vec<c_char> = vec![0; (length as usize) + 1];
    let native_result =
      unsafe { ffi::clipboard_get_text(buffer.as_mut_ptr(), buffer.len() as i32) };

    if native_result > 0 {
      let string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
      Some(string.to_string_lossy().to_string())
    } else {
      None
    }
  }

  fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> anyhow::Result<()> {
    let string = CString::new(text)?;
    let native_result = unsafe { ffi::clipboard_set_text(string.as_ptr()) };
    if native_result > 0 {
      Ok(())
    } else {
      Err(CocoaClipboardError::SetOperationFailed().into())
    }
  }

  fn set_image(
    &self,
    image_path: &std::path::Path,
    _: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    if !image_path.exists() || !image_path.is_file() {
      return Err(CocoaClipboardError::ImageNotFound(image_path.to_path_buf()).into());
    }

    let path = CString::new(image_path.to_string_lossy().to_string())?;
    let native_result = unsafe { ffi::clipboard_set_image(path.as_ptr()) };

    if native_result > 0 {
      Ok(())
    } else {
      Err(CocoaClipboardError::SetOperationFailed().into())
    }
  }

  fn set_html(
    &self,
    html: &str,
    fallback_text: Option<&str>,
    _: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    let html_string = CString::new(html)?;
    let fallback_string = CString::new(fallback_text.unwrap_or_default())?;
    let fallback_ptr = if fallback_text.is_some() {
      fallback_string.as_ptr()
    } else {
      std::ptr::null()
    };

    let native_result = unsafe { ffi::clipboard_set_html(html_string.as_ptr(), fallback_ptr) };
    if native_result > 0 {
      Ok(())
    } else {
      Err(CocoaClipboardError::SetOperationFailed().into())
    }
  }
}

#[derive(Error, Debug)]
pub enum CocoaClipboardError {
  #[error("clipboard set operation failed")]
  SetOperationFailed(),

  #[error("image not found: `{0}`")]
  ImageNotFound(PathBuf),
}
