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
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::Stdio,
};

use crate::{Clipboard, ClipboardOperationOptions, ClipboardOptions};
use anyhow::Result;
use log::{error, warn};
use std::process::Command;
use thiserror::Error;
use wait_timeout::ChildExt;

pub(crate) struct WaylandFallbackClipboard {
    command_timeout: u64,
}

impl WaylandFallbackClipboard {
    pub fn new(options: ClipboardOptions) -> Result<Self> {
        // Make sure wl-paste and wl-copy are available
        if Command::new("wl-paste").arg("--version").output().is_err() {
            error!("unable to call 'wl-paste' binary, please install the wl-clipboard package.");
            return Err(WaylandFallbackClipboardError::MissingWLClipboard().into());
        }
        if Command::new("wl-copy").arg("--version").output().is_err() {
            error!("unable to call 'wl-copy' binary, please install the wl-clipboard package.");
            return Err(WaylandFallbackClipboardError::MissingWLClipboard().into());
        }

        // Try to connect to the wayland display
        let wayland_socket = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let wayland_display = if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
                display
            } else {
                warn!("Could not determine wayland display from WAYLAND_DISPLAY env variable, falling back to 'wayland-0'");
                warn!("Note that this might not work on some systems.");
                "wayland-0".to_string()
            };

            PathBuf::from(runtime_dir).join(wayland_display)
        } else {
            error!(
                "environment variable XDG_RUNTIME_DIR is missing, can't initialize the clipboard"
            );
            return Err(WaylandFallbackClipboardError::MissingEnvVariable().into());
        };
        if UnixStream::connect(wayland_socket).is_err() {
            error!("failed to connect to Wayland display");
            return Err(WaylandFallbackClipboardError::ConnectionFailed().into());
        }

        Ok(Self {
            command_timeout: options.wayland_command_timeout_ms,
        })
    }
}

impl Clipboard for WaylandFallbackClipboard {
    fn get_text(&self, _: &ClipboardOperationOptions) -> Option<String> {
        let mut command = Command::new("wl-paste");
        command.arg("--no-newline");

        let mut output = String::new();

        self.invoke_command_with_timeout(
            command,
            None,
            Some(&mut output),
            ClipboardAction::Get,
        )
        .map(|_| output)
        .ok()
    }

    fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> anyhow::Result<()> {
        // NOTE: Without explicit MIME type, wl-copy's auto-detection can give unexpected results
        // making the text not paste-able in some programs.
        let mut command = Command::new("wl-copy");
        command.arg("--type").arg("text/plain;charset=utf-8");

        self.invoke_command_with_timeout(
            command,
            Some(text.as_bytes()),
            None,
            ClipboardAction::Set,
        )
    }

    fn set_image(
        &self,
        image_path: &std::path::Path,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        if !image_path.exists() || !image_path.is_file() {
            return Err(
                WaylandFallbackClipboardError::ImageNotFound(image_path.to_path_buf()).into(),
            );
        }

        // Load the image data
        let mut file = std::fs::File::open(image_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mut command = Command::new("wl-copy");
        command.arg("--type").arg("image/png");

        self.invoke_command_with_timeout(
            command,
            Some(&data),
            None,
            ClipboardAction::Set,
        )
    }

    fn set_html(
        &self,
        html: &str,
        _fallback_text: Option<&str>,
        _: &ClipboardOperationOptions,
    ) -> anyhow::Result<()> {
        let mut command = Command::new("wl-copy");
        command.arg("--type").arg("text/html");

        self.invoke_command_with_timeout(
            command,
            Some(html.as_bytes()),
            None,
            ClipboardAction::Set,
        )
    }
}

impl WaylandFallbackClipboard {
    fn invoke_command_with_timeout(
        &self,
        mut command: Command,
        input_data: Option<&[u8]>,
        output_buffer: Option<&mut String>,
        action: ClipboardAction,
    ) -> Result<()> {
        let timeout = std::time::Duration::from_millis(self.command_timeout);

        if input_data.is_some() {
            command.stdin(Stdio::piped());
        }

        if output_buffer.is_some() {
            command.stdout(Stdio::piped());
        }

        let name = command.get_program().to_string_lossy().into_owned();

        // Spawn the command upfront so we can stream input or capture output as needed
        let mut child = command.spawn().map_err(|err| {
            error!("could not invoke '{name}': {err}");
            action.into_error()
        })?;

        // Provide stdin payload if any
        if let Some(data) = input_data {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Unable to open stdin"))?;
            stdin.write_all(data)?;
        }

        // Monitor the child with a timeout
        match child.wait_timeout(timeout) {
            Ok(Some(status)) if status.success() => {}
            Ok(Some(_)) => {
                error!("error, {name} exited with non-zero exit code");
                let _ = child.wait();
                return Err(action.into_error().into());
            }
            Ok(None) => {
                error!("error, {name} has timed-out, killing the process");
                if child.kill().is_err() {
                    error!("unable to kill {name}");
                }
                let _ = child.wait();
                return Err(action.into_error().into());
            }
            Err(err) => {
                error!("error while executing '{name}': {err}");
                if child.kill().is_err() {
                    error!("unable to kill {name}");
                }
                let _ = child.wait();
                return Err(action.into_error().into());
            }
        }

        // Command exited successfully, collect stdout if needed
        if let Some(buffer) = output_buffer {
            let mut stdout = child.stdout.take().ok_or_else(|| {
                error!("stdout not available for '{name}'");
                action.into_error()
            })?;
            buffer.clear();
            stdout.read_to_string(buffer)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ClipboardAction {
    Get,
    Set,
}

impl ClipboardAction {
    fn into_error(self) -> WaylandFallbackClipboardError {
        match self {
            ClipboardAction::Get => WaylandFallbackClipboardError::GetOperationFailed(),
            ClipboardAction::Set => WaylandFallbackClipboardError::SetOperationFailed(),
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum WaylandFallbackClipboardError {
    #[error("wl-clipboard binaries are missing")]
    MissingWLClipboard(),

    #[error("missing XDG_RUNTIME_DIR env variable")]
    MissingEnvVariable(),

    #[error("can't connect to Wayland display")]
    ConnectionFailed(),

    #[error("clipboard set operation failed")]
    SetOperationFailed(),

    #[error("clipboard get operation failed")]
    GetOperationFailed(),

    #[error("image not found: `{0}`")]
    ImageNotFound(PathBuf),
}
