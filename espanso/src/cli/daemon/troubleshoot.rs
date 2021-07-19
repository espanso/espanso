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

use std::process::{Child, Command};

use anyhow::{Result, bail};
use espanso_path::Paths;

use crate::cli::util::CommandExt;
use crate::cli::PathsOverrides;
use crate::config::ConfigLoadResult;
use crate::error_eprintln;
use crate::preferences::Preferences;

pub fn launch_troubleshoot(
  paths_overrides: &PathsOverrides,
) -> Result<TroubleshootGuard> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
  command.args(&["modulo", "troubleshoot"]);
  command.with_paths_overrides(paths_overrides);

  let child = command.spawn()?;

  Ok(TroubleshootGuard::new(child))
}

pub fn launch_troubleshoot_blocking(paths_overrides: &PathsOverrides) -> Result<()> {
  let mut guard = launch_troubleshoot(paths_overrides)?;
  guard.wait()?;
  Ok(())
}

pub struct TroubleshootGuard {
  child: Child,
}

impl TroubleshootGuard {
  pub fn new(child: Child) -> Self {
    Self { child }
  }
  pub fn wait(&mut self) -> Result<()> {
    self.child.wait()?;
    Ok(())
  }
}

impl Drop for TroubleshootGuard {
  fn drop(&mut self) {
    let _ = self.child.kill();
  }
}

pub fn load_config_or_troubleshoot(
  paths: &Paths,
  paths_overrides: &PathsOverrides,
  troubleshoot_non_fatal: bool,
) -> Result<(ConfigLoadResult, Option<TroubleshootGuard>)> {
  match crate::load_config(&paths.config, &paths.packages) {
    Ok(load_result) => {
      let mut troubleshoot_handle = None;
      
      if troubleshoot_non_fatal && !load_result.non_fatal_errors.is_empty() {
        let preferences =
          crate::preferences::get_default(&paths.runtime).expect("unable to get preferences");

        if preferences.should_display_troubleshoot_for_non_fatal_errors() {
          match launch_troubleshoot(paths_overrides) {
            Ok(handle) => {
              troubleshoot_handle = Some(handle);
            },
            Err(err) => {
              error_eprintln!("unable to launch troubleshoot GUI: {}", err);
            }
          }
        }
      }
      Ok((load_result, troubleshoot_handle))
    }
    Err(fatal_err) => {
      launch_troubleshoot_blocking(paths_overrides)?;

      bail!("fatal error while loading config: {}", fatal_err);
    }
  }
}