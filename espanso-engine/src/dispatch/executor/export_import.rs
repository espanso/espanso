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

use crate::event::EventType;
use crate::{dispatch::Executor, event::Event};
use anyhow::Result;
use log::error;

pub trait ExportImportHandler {
    fn handle_export(&self) -> Result<()>;
    fn handle_import(&self) -> Result<()>;
}

pub struct ExportImportExecutor<'a> {
    handler: &'a dyn ExportImportHandler,
}

impl<'a> ExportImportExecutor<'a> {
    pub fn new(handler: &'a dyn ExportImportHandler) -> Self {
        Self { handler }
    }
}

impl Executor for ExportImportExecutor<'_> {
    fn execute(&self, event: &Event) -> bool {
        if matches!(&event.etype, EventType::ExportConfig) {
            if let Err(error) = self.handler.handle_export() {
                error!("export handler reported an error: {error:?}");
            }
            return true;
        } else if matches!(&event.etype, EventType::ImportConfig) {
            if let Err(error) = self.handler.handle_import() {
                error!("import handler reported an error: {error:?}");
            }
            return true;
        }

        false
    }
}
