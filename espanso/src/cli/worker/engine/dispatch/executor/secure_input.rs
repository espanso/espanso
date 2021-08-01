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
use std::process::Stdio;

use anyhow::{bail, Context};
use log::{error, info};

use crate::engine::dispatch::SecureInputManager;

pub struct SecureInputManagerAdapter {}

impl SecureInputManagerAdapter {
  pub fn new() -> Self {
    Self {}
  }
}

impl SecureInputManager for SecureInputManagerAdapter {
  fn display_secure_input_troubleshoot(&self) -> anyhow::Result<()> {
    // TODO: replace with actual URL
    // TODO: in the future, this might be a self-contained WebView window
    opener::open_browser("https://espanso.org/docs")?;
    Ok(())
  }

  fn launch_secure_input_autofix(&self) -> anyhow::Result<()> {
    let espanso_path = std::env::current_exe()?;
    let child = std::process::Command::new(espanso_path)
      .args(&["workaround", "secure-input"])
      .stdout(Stdio::piped())
      .spawn()
      .context("unable to spawn workaround process")?;
    let output = child.wait_with_output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    let error_str = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      info!(
        "Secure input workaround executed successfully: {}",
        output_str
      );
      Ok(())
    } else {
      error!("Secure input autofix reported error: {}", error_str);
      bail!("non-successful autofix status code");
    }
  }
}
