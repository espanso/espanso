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

use std::io::Read;

use crate::{Clipboard, ClipboardOperationOptions};
use log::error;
use wl_clipboard_rs::paste;

use super::fallback::WaylandFallbackClipboard;

pub(crate) struct WaylandNativeClipboard {
    fallback: WaylandFallbackClipboard,
}

impl WaylandNativeClipboard {
    pub fn new(
        fallback: WaylandFallbackClipboard,
    ) -> Self {
        Self { fallback }
    }
}

impl Clipboard for WaylandNativeClipboard {
    fn get_text(
        &self,
        _: &ClipboardOperationOptions,
    ) -> Option<String> {
        match paste::get_contents(
            paste::ClipboardType::Regular,
            paste::Seat::Unspecified,
            paste::MimeType::Text,
        ) {
            Ok((mut pipe, _mime)) => {
                let mut contents = String::new();
                match pipe.read_to_string(&mut contents) {
                    Ok(_) => Some(contents),
                    Err(err) => {
                        error!(
                            "failed to read clipboard pipe: {}",
                            err
                        );
                        None
                    }
                }
            }
            Err(
                paste::Error::NoSeats
                | paste::Error::ClipboardEmpty
                | paste::Error::NoMimeType,
            ) => None,
            Err(err) => {
                error!(
                    "failed to get clipboard contents: {}",
                    err
                );
                None
            }
        }
    }

    fn set_text(
        &self,
        text: &str,
        options: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        self.fallback.set_text(text, options)
    }

    fn set_image(
        &self,
        image_path: &std::path::Path,
        options: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        self.fallback.set_image(image_path, options)
    }

    fn set_html(
        &self,
        html: &str,
        fallback_text: Option<&str>,
        options: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        self.fallback
            .set_html(html, fallback_text, options)
    }
}
