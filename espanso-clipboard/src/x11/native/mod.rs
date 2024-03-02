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

use std::{
    ffi::{CStr, CString},
    io::Read,
    path::PathBuf,
};

use crate::{Clipboard, ClipboardOperationOptions};
use anyhow::Result;
use std::os::raw::c_char;
use thiserror::Error;

mod ffi;

pub struct X11NativeClipboard {}

impl X11NativeClipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Clipboard for X11NativeClipboard {
    fn get_text(&self, _: &ClipboardOperationOptions) -> Option<String> {
        let mut buffer: [c_char; 2048] = [0; 2048];
        let native_result =
            unsafe { ffi::clipboard_x11_get_text(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) };
        if native_result > 0 {
            let string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            Some(string.to_string_lossy().to_string())
        } else {
            None
        }
    }

    fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> anyhow::Result<()> {
        let string = CString::new(text)?;
        let native_result = unsafe { ffi::clipboard_x11_set_text(string.as_ptr()) };
        if native_result > 0 {
            Ok(())
        } else {
            Err(X11NativeClipboardError::SetOperationFailed().into())
        }
    }

    fn set_image(
        &self,
        image_path: &std::path::Path,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        if !image_path.exists() || !image_path.is_file() {
            return Err(X11NativeClipboardError::ImageNotFound(image_path.to_path_buf()).into());
        }

        // Load the image data
        let mut file = std::fs::File::open(image_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let native_result =
            unsafe { ffi::clipboard_x11_set_image(data.as_ptr(), data.len() as i32) };

        if native_result > 0 {
            Ok(())
        } else {
            Err(X11NativeClipboardError::SetOperationFailed().into())
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

        let native_result =
            unsafe { ffi::clipboard_x11_set_html(html_string.as_ptr(), fallback_ptr) };
        if native_result > 0 {
            Ok(())
        } else {
            Err(X11NativeClipboardError::SetOperationFailed().into())
        }
    }
}

#[derive(Error, Debug)]
pub enum X11NativeClipboardError {
    #[error("clipboard set operation failed")]
    SetOperationFailed(),

    #[error("image not found: `{0}`")]
    ImageNotFound(PathBuf),
}
