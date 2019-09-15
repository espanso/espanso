/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use std::process::{Command, Stdio};
use std::io::{Write};
use log::error;

pub struct LinuxClipboardManager {}

impl super::ClipboardManager for LinuxClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        let res = Command::new("xclip")
            .args(&["-o", "-sel", "clip"])
            .output();

        if let Ok(output) = res {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                return Some((*s).to_owned());
            }
        }

        None
    }

    fn set_clipboard(&self, payload: &str) {
        let res = Command::new("xclip")
            .args(&["-sel", "clip"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = res {
            let stdin = child.stdin.as_mut();

            if let Some(output) = stdin {
                let res = output.write_all(payload.as_bytes());

                if let Err(e) = res {
                    error!("Could not set clipboard: {}", e);
                }
            }
        }
    }
}

impl LinuxClipboardManager {
    pub fn new() -> LinuxClipboardManager {
        LinuxClipboardManager{}
    }
}