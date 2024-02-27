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

use std::{ffi::CString, path::PathBuf};

use crate::{Clipboard, ClipboardOperationOptions};
use anyhow::Result;
use log::error;
use thiserror::Error;
use widestring::{U16CStr, U16CString};

pub struct Win32Clipboard {}

impl Win32Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Clipboard for Win32Clipboard {
    fn get_text(&self, _: &ClipboardOperationOptions) -> Option<String> {
        let mut buffer: [u16; 2048] = [0; 2048];
        let native_result =
            unsafe { ffi::clipboard_get_text(buffer.as_mut_ptr(), (buffer.len() - 1) as i32) };
        if native_result > 0 {
            let string = unsafe { U16CStr::from_ptr_str(buffer.as_ptr()) };
            Some(string.to_string_lossy())
        } else {
            None
        }
    }

    fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> Result<()> {
        let string = U16CString::from_str(text)?;
        let native_result = unsafe { ffi::clipboard_set_text(string.as_ptr()) };
        if native_result > 0 {
            Ok(())
        } else {
            Err(Win32ClipboardError::SetOperationFailed().into())
        }
    }

    fn set_image(
        &self,
        image_path: &std::path::Path,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        if !image_path.exists() || !image_path.is_file() {
            return Err(Win32ClipboardError::ImageNotFound(image_path.to_path_buf()).into());
        }

        let path = U16CString::from_os_str(image_path.as_os_str())?;
        let native_result = unsafe { ffi::clipboard_set_image(path.as_ptr()) };

        if native_result > 0 {
            Ok(())
        } else {
            Err(Win32ClipboardError::SetOperationFailed().into())
        }
    }

    fn set_html(
        &self,
        html: &str,
        fallback_text: Option<&str>,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        let html_descriptor = generate_html_descriptor(html);
        let html_string = CString::new(html_descriptor)?;
        let fallback_string = U16CString::from_str(fallback_text.unwrap_or_default())?;
        let fallback_ptr = if fallback_text.is_some() {
            fallback_string.as_ptr()
        } else {
            std::ptr::null()
        };

        let native_result = unsafe { ffi::clipboard_set_html(html_string.as_ptr(), fallback_ptr) };
        if native_result > 0 {
            Ok(())
        } else {
            Err(Win32ClipboardError::SetOperationFailed().into())
        }
    }
}

fn generate_html_descriptor(html: &str) -> String {
    // In order to set the HTML clipboard, we have to create a prefix with a specific format
    // For more information, look here:
    // https://docs.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format
    // https://docs.microsoft.com/en-za/troubleshoot/cpp/add-html-code-clipboard
    let content = format!("<!--StartFragment-->{html}<!--EndFragment-->");

    let tokens = [
        "Version:0.9",
        "StartHTML:<<STR*#>",
        "EndHTML:<<END*#>",
        "StartFragment:<<SFG#*>",
        "EndFragment:<<EFG#*>",
        "<html>",
        "<body>",
        &content,
        "</body>",
        "</html>",
    ];

    let mut render = tokens.join("\r\n");

    // Now replace the placeholders with the actual positions
    render = render.replace(
        "<<STR*#>",
        &format!("{:0>8}", render.find("<html>").unwrap_or_default()),
    );
    render = render.replace("<<END*#>", &format!("{:0>8}", render.len()));
    render = render.replace(
        "<<SFG#*>",
        &format!(
            "{:0>8}",
            render.find("<!--StartFragment-->").unwrap_or_default() + "<!--StartFragment-->".len()
        ),
    );
    render = render.replace(
        "<<EFG#*>",
        &format!(
            "{:0>8}",
            render.find("<!--EndFragment-->").unwrap_or_default()
        ),
    );
    render
}

#[derive(Error, Debug)]
pub enum Win32ClipboardError {
    #[error("clipboard set operation failed")]
    SetOperationFailed(),

    #[error("image not found: `{0}`")]
    ImageNotFound(PathBuf),
}
