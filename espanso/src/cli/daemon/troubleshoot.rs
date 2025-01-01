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
use std::time::Duration;

use anyhow::{bail, Result};
use crossbeam::channel::Receiver;
use crossbeam::select;
use espanso_path::Paths;
use log::info;

use crate::cli::util::CommandExt;
use crate::cli::PathsOverrides;
use crate::config::ConfigLoadResult;
use crate::error_eprintln;
use crate::preferences::Preferences;

pub fn launch_troubleshoot(paths_overrides: &PathsOverrides) -> Result<TroubleshootGuard> {
  let espanso_exe_path = std::env::current_exe()?;
  let mut command = Command::new(espanso_exe_path.to_string_lossy().to_string());
  command.args(["modulo", "troubleshoot"]);
  command.with_paths_overrides(paths_overrides);

  let child = command.spawn()?;

  Ok(TroubleshootGuard::new(child))
}

pub struct TroubleshootGuard {
  child: Child,
}

impl TroubleshootGuard {
  pub fn new(child: Child) -> Self {
    Self { child }
  }
  #[allow(dead_code)]
  pub fn wait(&mut self) -> Result<()> {
    self.child.wait()?;
    Ok(())
  }
  pub fn try_wait(&mut self) -> Result<bool> {
    let result = self.child.try_wait()?;
    Ok(result.is_some())
  }
}

impl Drop for TroubleshootGuard {
  fn drop(&mut self) {
    let _ = self.child.kill();
  }
}

pub enum LoadResult {
  Correct(ConfigLoadResult),
  Warning(ConfigLoadResult, Option<TroubleshootGuard>),
  Fatal(TroubleshootGuard),
}

pub fn load_config_or_troubleshoot(paths: &Paths, paths_overrides: &PathsOverrides) -> LoadResult {
  match crate::load_config(&paths.config) {
    Ok(load_result) => {
      if load_result.non_fatal_errors.is_empty() {
        LoadResult::Correct(load_result)
      } else {
        let mut troubleshoot_handle = None;

        let preferences =
          crate::preferences::get_default(&paths.runtime).expect("unable to get preferences");

        if preferences.should_display_troubleshoot_for_non_fatal_errors() {
          match launch_troubleshoot(paths_overrides) {
            Ok(handle) => {
              troubleshoot_handle = Some(handle);
            }
            Err(err) => {
              error_eprintln!("unable to launch troubleshoot GUI: {}", err);
            }
          }
        }

        LoadResult::Warning(load_result, troubleshoot_handle)
      }
    }
    Err(_) => LoadResult::Fatal(
      launch_troubleshoot(paths_overrides).expect("unable to launch troubleshoot GUI"),
    ),
  }
}

pub fn load_config_or_troubleshoot_until_config_is_correct_or_abort(
  paths: &Paths,
  paths_overrides: &PathsOverrides,
  watcher_receiver: Receiver<()>,
) -> Result<(ConfigLoadResult, Option<TroubleshootGuard>)> {
  let mut _troubleshoot_guard = None;

  loop {
    // If the loading process is fatal, we keep showing the troubleshooter until
    // either the config is correct or the user aborts by closing the troubleshooter
    _troubleshoot_guard = match load_config_or_troubleshoot(paths, paths_overrides) {
      LoadResult::Correct(result) => return Ok((result, None)),
      LoadResult::Warning(result, guard) => return Ok((result, guard)),
      LoadResult::Fatal(guard) => Some(guard),
    };

    loop {
      select! {
        recv(watcher_receiver) -> _ => {
          info!("config change detected, reloading configs...");

          break
        },
        default(Duration::from_millis(500)) => {
          if let Some(guard) = &mut _troubleshoot_guard {
            if let Ok(ended) = guard.try_wait() {
              if ended {
                bail!("user aborted troubleshooter");
              }
            } else {
              bail!("unable to wait for troubleshooter");
            }
          } else {
            bail!("no troubleshoot guard found");
          }
        }
      }
    }
  }
}
