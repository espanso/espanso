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
use std::thread;

use espanso_engine::dispatch::ManagementPanelHandler;

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
        let config_dir = self.config_dir.clone();
        let runtime_dir = self.runtime_dir.clone();

        // Run the GUI on a dedicated thread so it doesn't block the engine
        thread::spawn(move || {
            espanso_gui::run(
                Some(config_dir),
                Some(runtime_dir),
                espanso_gui::app::Module::MatchManager,
            );
        });
    }
}
