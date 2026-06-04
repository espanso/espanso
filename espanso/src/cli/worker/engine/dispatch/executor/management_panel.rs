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
        // Kill any previously running GUI instance first
        self.kill_child();

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

        // On macOS, when packaged as .app with LSUIElement=1, the subprocess
        // inherits the same Info.plist and can't open GUI windows properly.
        // Use `open -a AppBundle.app --args` to let macOS handle the launch.
        #[cfg(target_os = "macos")]
        let spawn_result = {
            // Detect if we're inside an .app bundle
            let is_bundle = exe.to_string_lossy().contains(".app/Contents/MacOS/");
            if is_bundle {
                // Find the .app bundle root
                let app_bundle = exe
                    .ancestors()
                    .find(|p| p.extension().map(|e| e == "app").unwrap_or(false));
                if let Some(bundle) = app_bundle {
                    info!("detected app bundle, using 'open -a' for GUI launch");
                    Command::new("open")
                        .arg("-a")
                        .arg(bundle)
                        .arg("--args")
                        .arg("gui")
                        .arg(format!("--config_dir={}", self.config_dir.display()))
                        .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
                        .spawn()
                } else {
                    Command::new(&exe)
                        .arg("gui")
                        .arg(format!("--config_dir={}", self.config_dir.display()))
                        .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
                        .spawn()
                }
            } else {
                Command::new(&exe)
                    .arg("gui")
                    .arg(format!("--config_dir={}", self.config_dir.display()))
                    .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
                    .spawn()
            }
        };

        #[cfg(not(target_os = "macos"))]
        let spawn_result = Command::new(&exe)
            .arg("gui")
            .arg(format!("--config_dir={}", self.config_dir.display()))
            .arg(format!("--runtime_dir={}", self.runtime_dir.display()))
            .spawn();

        match spawn_result {
            Ok(child) => {
                info!("espanso gui subprocess spawned (pid {})", child.id());
                *self.child.lock().unwrap() = Some(child);
            }
            Err(e) => {
                error!("failed to spawn espanso gui subprocess: {}", e);
            }
        }
    }
}
