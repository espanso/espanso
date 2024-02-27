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

use anyhow::Result;
use log::error;

use crate::{
    dispatch::Executor,
    event::{Event, EventType},
};

pub trait SecureInputManager {
    fn display_secure_input_troubleshoot(&self) -> Result<()>;
    fn launch_secure_input_autofix(&self) -> Result<()>;
}

pub struct SecureInputExecutor<'a> {
    manager: &'a dyn SecureInputManager,
}

impl<'a> SecureInputExecutor<'a> {
    pub fn new(manager: &'a dyn SecureInputManager) -> Self {
        Self { manager }
    }
}

impl<'a> Executor for SecureInputExecutor<'a> {
    fn execute(&self, event: &Event) -> bool {
        if let EventType::DisplaySecureInputTroubleshoot = &event.etype {
            if let Err(error) = self.manager.display_secure_input_troubleshoot() {
                error!("unable to display secure input troubleshoot: {}", error);
            }
            return true;
        } else if let EventType::LaunchSecureInputAutoFix = &event.etype {
            if let Err(error) = self.manager.launch_secure_input_autofix() {
                error!("unable to launch secure input autofix: {}", error);
            }
            return true;
        }

        false
    }
}
