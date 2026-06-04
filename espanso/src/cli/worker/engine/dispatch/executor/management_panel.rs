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

use espanso_engine::dispatch::ManagementPanelHandler;
use log::{error, info};

pub struct ManagementPanelHandlerAdapter {
    config_dir: PathBuf,
    runtime_dir: PathBuf,
}

impl ManagementPanelHandlerAdapter {
    pub fn new(config_dir: PathBuf, runtime_dir: PathBuf) -> Self {
        Self {
            config_dir,
            runtime_dir,
        }
    }
}

impl ManagementPanelHandler for ManagementPanelHandlerAdapter {
    fn open_management_panel(&self) {
        // macOS requires winit EventLoop on the main thread, so we spawn
        // a separate process (same binary, `espanso gui` subcommand).
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                error!("cannot find current exe path: {}", e);
                return;
            }
        };

        info!(
            "spawning espanso gui subprocess: {} gui --config_dir={}",
            exe.display(),
            self.config_dir.display(),
        );

        match Command::new(&exe)
            .arg("gui")
            .arg(format!("--config_dir={}", self.config_dir.display()))
            .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
            .spawn()
        {
            Ok(_child) => {
                info!("espanso gui subprocess spawned successfully");
            }
            Err(e) => {
                error!("failed to spawn espanso gui subprocess: {}", e);
            }
        }
    }
}
