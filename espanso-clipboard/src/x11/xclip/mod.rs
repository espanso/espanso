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

use anyhow::bail;
use log::error;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::{Clipboard, ClipboardOperationOptions};

pub struct XClipClipboard {
    is_xclip_available: bool,
}

impl XClipClipboard {
    pub fn new() -> Self {
        let command = Command::new("xclip").arg("-h").output();
        let is_xclip_available = command
            .map(|output| output.status.success())
            .unwrap_or(false);

        Self { is_xclip_available }
    }
}

impl Clipboard for XClipClipboard {
    fn get_text(&self, _: &ClipboardOperationOptions) -> Option<String> {
        if !self.is_xclip_available {
            error!("attempted to use XClipClipboard, but `xclip` command can't be called");
            return None;
        }

        match Command::new("xclip").args(["-o", "-sel", "clip"]).output() {
            Ok(output) => {
                if output.status.success() {
                    let s = String::from_utf8_lossy(&output.stdout);
                    return Some(s.to_string());
                }
            }
            Err(error) => {
                error!("xclip reported an error: {}", error);
            }
        }

        None
    }

    fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> anyhow::Result<()> {
        if !self.is_xclip_available {
            bail!("attempted to use XClipClipboard, but `xclip` command can't be called");
        }

        let mut child = Command::new("xclip")
            .args(["-sel", "clip"])
            .stdin(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.as_mut();
        if let Some(input) = stdin {
            input.write_all(text.as_bytes())?;
            child.wait()?;
        }

        Ok(())
    }

    fn set_image(
        &self,
        image_path: &std::path::Path,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        if !self.is_xclip_available {
            bail!("attempted to use XClipClipboard, but `xclip` command can't be called");
        }

        let extension = image_path.extension();
        let mime = match extension {
            Some(ext) => {
                let ext = ext.to_string_lossy().to_lowercase();
                match ext.as_ref() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "svg" => "image/svg",
                    _ => "image/png",
                }
            }
            None => "image/png",
        };

        let image_path = image_path.to_string_lossy();

        Command::new("xclip")
            .args(["-selection", "clipboard", "-t", mime, "-i", &image_path])
            .spawn()?;

        Ok(())
    }

    fn set_html(
        &self,
        html: &str,
        _: Option<&str>,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        if !self.is_xclip_available {
            bail!("attempted to use XClipClipboard, but `xclip` command can't be called");
        }

        let mut child = Command::new("xclip")
            .args(["-sel", "clip", "-t", "text/html"])
            .stdin(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.as_mut();
        if let Some(input) = stdin {
            input.write_all(html.as_bytes())?;
            child.wait()?;
        }

        Ok(())
    }
}
