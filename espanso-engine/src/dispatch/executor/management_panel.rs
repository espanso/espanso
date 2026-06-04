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

use crate::dispatch::Executor;
use crate::event::{Event, EventType};

pub trait ManagementPanelHandler {
    fn open_management_panel(&self);
}

pub struct ManagementPanelExecutor<'a> {
    handler: &'a dyn ManagementPanelHandler,
}

impl<'a> ManagementPanelExecutor<'a> {
    pub fn new(handler: &'a dyn ManagementPanelHandler) -> Self {
        Self { handler }
    }
}

impl Executor for ManagementPanelExecutor<'_> {
    fn execute(&self, event: &Event) -> bool {
        if matches!(event.etype, EventType::OpenManagementPanel) {
            self.handler.open_management_panel();
            return true;
        }
        false
    }
}
