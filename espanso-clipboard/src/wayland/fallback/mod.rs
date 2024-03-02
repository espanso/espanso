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
      error!("environment variable XDG_RUNTIME_DIR is missing, can't initialize the clipboard");
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
    let timeout = std::time::Duration::from_millis(self.command_timeout);
    match Command::new("wl-paste")
      .arg("--no-newline")
      .stdout(Stdio::piped())
      .spawn()
    {
      Ok(mut child) => match child.wait_timeout(timeout) {
        Ok(status_code) => {
          if let Some(status) = status_code {
            if status.success() {
              if let Some(mut io) = child.stdout {
                let mut output = Vec::new();
                io.read_to_end(&mut output).ok()?;
                Some(String::from_utf8_lossy(&output).to_string())
              } else {
                None
              }
            } else {
              error!("error, wl-paste exited with non-zero exit code");
              None
            }
          } else {
            error!("error, wl-paste has timed-out, killing the process");
            if child.kill().is_err() {
              error!("unable to kill wl-paste");
            }
            None
          }
        }
        Err(err) => {
          error!("error while executing 'wl-paste': {}", err);
          None
        }
      },
      Err(err) => {
        error!("could not invoke 'wl-paste': {}", err);
        None
      }
    }
  }

  fn set_text(&self, text: &str, _: &ClipboardOperationOptions) -> anyhow::Result<()> {
    self.invoke_command_with_timeout(&mut Command::new("wl-copy"), text.as_bytes(), "wl-copy")
  }

  fn set_image(
    &self,
    image_path: &std::path::Path,
    _: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    if !image_path.exists() || !image_path.is_file() {
      return Err(WaylandFallbackClipboardError::ImageNotFound(image_path.to_path_buf()).into());
    }

    // Load the image data
    let mut file = std::fs::File::open(image_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    self.invoke_command_with_timeout(
      Command::new("wl-copy").arg("--type").arg("image/png"),
      &data,
      "wl-copy",
    )
  }

  fn set_html(
    &self,
    html: &str,
    _fallback_text: Option<&str>,
    _: &ClipboardOperationOptions,
  ) -> anyhow::Result<()> {
    self.invoke_command_with_timeout(
      Command::new("wl-copy").arg("--type").arg("text/html"),
      html.as_bytes(),
      "wl-copy",
    )
  }
}

impl WaylandFallbackClipboard {
  fn invoke_command_with_timeout(
    &self,
    command: &mut Command,
    data: &[u8],
    name: &str,
  ) -> Result<()> {
    let timeout = std::time::Duration::from_millis(self.command_timeout);
    match command.stdin(Stdio::piped()).spawn() {
      Ok(mut child) => {
        if let Some(stdin) = child.stdin.as_mut() {
          stdin.write_all(data)?;
        }
        match child.wait_timeout(timeout) {
          Ok(status_code) => {
            if let Some(status) = status_code {
              if status.success() {
                Ok(())
              } else {
                error!("error, {} exited with non-zero exit code", name);
                Err(WaylandFallbackClipboardError::SetOperationFailed().into())
              }
            } else {
              error!("error, {} has timed-out, killing the process", name);
              if child.kill().is_err() {
                error!("unable to kill {}", name);
              }
              Err(WaylandFallbackClipboardError::SetOperationFailed().into())
            }
          }
          Err(err) => {
            error!("error while executing '{}': {}", name, err);
            Err(WaylandFallbackClipboardError::SetOperationFailed().into())
          }
        }
      }
      Err(err) => {
        error!("could not invoke '{}': {}", name, err);
        Err(WaylandFallbackClipboardError::SetOperationFailed().into())
      }
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

  #[error("image not found: `{0}`")]
  ImageNotFound(PathBuf),
}
