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

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use espanso_engine::ManagementPanelCallback;
use log::{error, info};

pub struct ManagementPanelHandlerAdapter {
    config_dir: PathBuf,
    runtime_dir: PathBuf,
    /// Track the spawned child so we can kill it on exit (avoids zombie processes).
    child: Mutex<Option<std::process::Child>>,
}

impl ManagementPanelHandlerAdapter {
    pub fn new(config_dir: PathBuf, runtime_dir: PathBuf) -> Self {
        Self {
            config_dir,
            runtime_dir,
            child: Mutex::new(None),
        }
    }

    /// Kill the GUI child process if it's still running.
    pub fn kill_child(&self) {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            info!("killing espanso gui subprocess (pid {})", child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for ManagementPanelHandlerAdapter {
    fn drop(&mut self) {
        self.kill_child();
    }
}

impl ManagementPanelCallback for ManagementPanelHandlerAdapter {
    fn open_management_panel(&self) {
        info!("=== ManagementPanelHandlerAdapter::open_management_panel() called ===");
        info!(
            "config_dir={}, runtime_dir={}",
            self.config_dir.display(),
            self.runtime_dir.display()
        );

        // Kill any previously running GUI instance first
        self.kill_child();

        let exe = match std::env::current_exe() {
            Ok(p) => {
                info!("current_exe = {}", p.display());
                p
            }
            Err(e) => {
                error!("cannot find current exe path: {}", e);
                return;
            }
        };

        // Always spawn the binary directly. Even inside an .app bundle with
        // LSUIElement=1, direct Command::new() spawns a plain process that
        // bypasses LaunchServices — so the Info.plist background-only flag
        // does NOT apply. Using `open -a` would go through LaunchServices
        // which reads LSUIElement=1 and prevents GUI windows from appearing.
        // NOTE: `--config_dir` and `--runtime_dir` are top-level (global) options,
        // so they MUST appear BEFORE the `gui` subcommand. Placing them after `gui`
        // makes clap parse them with the `gui` subcommand parser (which only knows
        // the `module` positional), causing an UnknownArgument error and an
        // immediate exit — the window never appears even though spawn() succeeds.
        info!(
            "spawning: {} --config_dir={} --runtime_dir={} gui",
            exe.display(),
            self.config_dir.display(),
            self.runtime_dir.display()
        );

        let spawn_result = Command::new(&exe)
            .arg(format!("--config_dir={}", self.config_dir.display()))
            .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
            .arg("gui")
            .spawn();

        match spawn_result {
            Ok(child) => {
                info!(
                    "SUCCESS: espanso gui subprocess spawned (pid {})",
                    child.id()
                );
                *self.child.lock().unwrap() = Some(child);
            }
            Err(e) => {
                error!("FAILED to spawn espanso gui subprocess: {}", e);
            }
        }
    }
}
