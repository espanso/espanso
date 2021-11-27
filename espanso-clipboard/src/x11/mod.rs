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

use crate::{Clipboard, ClipboardOperationOptions};

mod native;
mod xclip;

pub(crate) struct X11Clipboard {
  native_backend: native::X11NativeClipboard,
  xclip_backend: xclip::XClipClipboard,
}

impl X11Clipboard {
  pub fn new() -> Result<Self> {
    Ok(Self {
      native_backend: native::X11NativeClipboard::new()?,
      xclip_backend: xclip::XClipClipboard::new(),
    })
  }
}

impl Clipboard for X11Clipboard {
  fn get_text(&self, options: &ClipboardOperationOptions) -> Option<String> {
    if options.use_xclip_backend {
      self.xclip_backend.get_text(options)
    } else {
      self.native_backend.get_text(options)
    }
  }

  fn set_text(&self, text: &str, options: &ClipboardOperationOptions) -> anyhow::Result<()> {
    if options.use_xclip_backend {
      self.xclip_backend.set_text(text, options)
    } else {
      self.native_backend.set_text(text, options)
    }
  }

  fn set_image(
    &self,
    image_path: &std::path::Path,
    options: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    if options.use_xclip_backend {
      self.xclip_backend.set_image(image_path, options)
    } else {
      self.native_backend.set_image(image_path, options)
    }
  }

  fn set_html(
    &self,
    html: &str,
    fallback_text: Option<&str>,
    options: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    if options.use_xclip_backend {
      self.xclip_backend.set_html(html, fallback_text, options)
    } else {
      self.native_backend.set_html(html, fallback_text, options)
    }
  }
}
